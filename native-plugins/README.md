# Lime Native — standalone VST3 suite

Twelve independent VST3 plugins for **any VST3-compatible DAW**. LimeShot does not need to be installed or running.
Each bundle contains its own native editor and statically linked DSP. No server, account, internet connection,
LimeShot project or companion application is needed.

| Plugin | What it does |
| --- | --- |
| **LimeSynth** | 128 voices; sine, square, saw and triangle; detune/octave, ADSR, resonant low-/band-/high-pass with key tracking, filter envelope, LFO/vibrato |
| **Lime Sampler** | Stereo WAV playback, embedded sample recall, transpose/root key, trim, reverse, gated/one-shot, attack/decay/sustain/release, low-pass filter, loop |
| **Lime Utility** | Gain, stereo width and polarity |
| **Lime EQ** | Low shelf, parametric mid and high shelf |
| **Lime Filter** | Resonant low-, band- and high-pass, wet/dry mix |
| **Lime Delay** | Stereo feedback delay, tone/spread, wet/dry, host-tempo sync |
| **Lime Room** | Algorithmic stereo reverb, pre-delay, size/damping/width and wet/dry |
| **Lime Comp** | Threshold/ratio, attack/release, makeup, optional stereo external sidechain |
| **Lime Gate** | Threshold, attack, hold, release and range |
| **Lime Drive** | Soft clip, hard clip and wave folding; tone, mix and output |
| **Lime Chorus** | Chorus, flanger and phaser; rate, depth, feedback, stereo spread and mix |
| **Lime Limiter** | Stereo-linked, zero-lookahead sample-peak limiter with ceiling/release and gain-reduction metering |

Editors use LimeShot's Lime Dark colors, flat controls, original painter artwork and the existing
Inter/JetBrains Mono fonts. Each plugin has its own proportions, layout and accent: an oscillator/envelope
instrument, waveform/keyboard sampler, compact utility, graph-led EQ, filter tiles, segmented delay,
wireframe room, compressor tiles and wide limiter history. Layout references based on Image-Line's official manuals are recorded in the repository at
`docs/research/lime-native-ui.md`.

Live output/meters, factory presets and `.limepreset` import/export are included. The EQ graph uses the actual
Eq3 frequency response and supports node dragging; the filter curve is a response guide. Neither is a spectrum
analyzer. The limiter history displays measured input/output levels and gain reduction.
Drag knobs vertically; Shift gives fine adjustment, double-click resets, wheel adjusts, and right-click accepts a
numeric value or named choice. Parameter changes use host automation gestures and smoothing. The ON/BYPASS button
controls host bypass (effects pass input unchanged; instruments become silent while notes keep their timing).

## Ready-to-use bundles and installation

`dist/` contains the generated plugins and a ZIP package for the platform/architecture used to build them. The development Mac's
bundles are **macOS Apple Silicon (arm64)** and are ad-hoc signed, not notarized. They are not Windows/Linux binaries
or Intel/universal Mac binaries. The source/build workflow supports those targets; runtime validation there is pending.

Copy the individual `.vst3` bundles, preserving their contents, to your user plugin directory, then rescan in your DAW:

- macOS: `~/Library/Audio/Plug-Ins/VST3/`
- Windows: `C:\Program Files\Common Files\VST3\` (or a custom VST3 folder supported by your DAW)
- Linux: `~/.vst3/`

FL Studio, REAPER, Cubase, Ableton Live and Bitwig support VST3. Logic Pro requires AU and cannot load these VST3
bundles directly. VST2/AU/CLAP export is not included in this suite. The build script never installs plugins or
changes your DAW's settings automatically.

## Build

Requires **Rust 1.95+**, Python 3, and your platform's native build tools. This is a separate Cargo workspace;
its dependencies and `Cargo.lock` do not change the DAW's workspace. Always invoke its manifest or run from here.

```sh
# From the repository root: builds all twelve, bundles and collects them in native-plugins/dist/.
python3 native-plugins/build.py

# From native-plugins/: bundle selected plugins, or build a different installed Rust target.
cargo xtask bundle -p lime-synth -p lime-limiter --release
python3 build.py --target x86_64-apple-darwin
# Allocation-checked development builds go to target/debug-bundled/.
python3 build.py --debug
```

Build Windows on Windows and Linux on Linux; a Rust target alone does not provide the OS's linker/GUI libraries.
Linux needs X11/OpenGL development libraries: `libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev
libxkbcommon-dev` (Debian/Ubuntu). Dialog dependencies use rfd's XDG portal backend. CI builds native bundles on
macOS, Windows and Linux and uploads them separately.

## Verify

```sh
cargo test --manifest-path native-plugins/Cargo.toml --workspace
cargo clippy --manifest-path native-plugins/Cargo.toml --workspace --all-targets -- -D warnings
cargo fmt --manifest-path native-plugins/Cargo.toml --all --check
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example verify -- native-plugins/dist
cargo run --manifest-path native-plugins/Cargo.toml -p lime-native --example ui_shot -- out/native-plugins
```

The `verify` program loads the **exported binaries** through LimeShot's VST3 host, checking actual ABI loading,
parameter/state recall, editor creation, timestamped MIDI onset and automation at 44.1/48/96 kHz and buffer sizes
1/64/333/1024. DSP tests compare all seven effects against the DAW's processors at multiple buffer sizes and rates;
test sustain, repeated-key polyphony/stealing/choke, held-note automation, sample recall after file deletion,
sidechain ducking, delay tempo sync and limiter ceiling/bypass. Debug plugins enable realtime allocation assertions.

An independent test host, [pluginval](https://github.com/Tracktion/pluginval), can check each exported bundle:

```sh
pluginval --strictness-level 5 --validate '/absolute/path/to/LimeSynth.vst3'
# On a headless runner, add --skip-gui-tests.
```

Recorded validation and platform limits are in [VALIDATION.md](VALIDATION.md). Passing a validator does not establish
hands-on compatibility with every DAW/version.

## Sampler and presets

Use Load or drop a WAV, MP3, FLAC, Ogg Vorbis or AIFF file to load it. Hold the waveform or keyboard to audition; right-click a key to set
the root note. Drag the colored waveform handles to trim the playable region. The shared LimeShot audio decoder is used here. Samples
are limited to **6 million stereo frames** (about 125 seconds at 48 kHz) to keep embedded host state below 64 MiB.
The plugin saves decoded audio inside host state; moving/deleting the source file does not break a saved session.
Loading happens during non-realtime state restoration/activation and resets sounding notes. No disk access,
allocation or locking occurs during audio processing. An invalid embedded sample becomes an empty, silent sampler.

Factory and `.limepreset` presets share LimeShot's existing preset implementation. Sampler `.limepreset` files
change playback settings, preserving the loaded sample and root key; host projects/presets save all parameters
and embedded sample data. Hosts may also save their own `.vstpreset` files from plugin state.

Synth envelope/oscillator/filter/LFO controls and sampler controls update held voices. Pitch bend is ±2 semitones;
sustain pedal, all-notes-off, all-sound-off and note choke are supported. Instruments have 128 voices and a stereo
output. Effects have stereo input/output; Lime Comp additionally exposes an optional stereo key input. The limiter
limits sample peaks; it does not provide true-peak oversampling or lookahead.

## Source layout

`lime-*/` are small VST3 entry points with stable class IDs. `shared/` implements parameters, MIDI, editor and the
adapter to `../crates/limeshot-{dsp,engine,sequencer,core,project}`. `xtask/` bundles native binaries with nice-plug;
`build.py` collects them into `dist/`. No DSP is copied from another plugin or proprietary application. Keep the
parameter array order and class IDs stable: hosts identify existing automation and sessions with them.

See [THIRD_PARTY.md](THIRD_PARTY.md) for dependencies and licenses.

LimeShot clip playback, additive channel/note pan and native metering use versioned noncommercial SysEx extensions.
Other VST3 hosts continue to use ordinary MIDI, note expressions and the standard stereo key bus.
Lime EQ appends hidden track-mode controls for the host inline EQ; existing insert parameter IDs/ranges stay stable.


The local egui-baseview 0.9.0 patch in `vendor/egui-baseview` binds its renderer’s GL context before
destroying the painter. This prevents editor closure from deleting another host/editor’s GL objects.
Upstream licenses and patch rationale are retained alongside the source.
