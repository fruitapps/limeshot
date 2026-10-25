<h1 align="center">LimeShot</h1>

<p align="center"><strong>The pattern-based DAW, in Rust.</strong></p>

<p align="center">
  <a href="https://github.com/fruitapps/limeshot/actions/workflows/ci.yml"><img src="https://github.com/fruitapps/limeshot/actions/workflows/ci.yml/badge.svg?branch=main" alt="Main branch CI status"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Rust-1.88%2B-b7410e?style=flat-square&amp;logo=rust" alt="DAW declared Rust minimum: 1.88"></a>
  <a href="#project-status"><img src="https://img.shields.io/badge/status-early%20alpha-d69e2e?style=flat-square" alt="Status: early alpha"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square" alt="License: MIT or Apache-2.0"></a>
  <a href="https://github.com/fruitapps/limeshot/stargazers"><img src="https://img.shields.io/github/stars/fruitapps/limeshot?style=flat-square" alt="GitHub stars"></a>
</p>

<p align="center">
  <a href="docs/screenshots/current/workspace.png"><img src="docs/screenshots/current/workspace.png" alt="Current LimeShot workspace rendered from main: Channel Rack and Playlist" width="900"></a>
</p>

<p align="center">
  <a href="#build-from-source">Build from source</a> ·
  <a href="#lime-native-standalone-vst3-suite">Standalone VST3 suite</a> ·
  <a href="docs/roadmap/features.md">Feature inventory</a> ·
  <a href="docs/roadmap/known-limitations.md">Known limitations</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

LimeShot is an open-source, offline-first digital audio workstation built around the pattern workflow of FL Studio:
program beats in the Channel Rack, write melodies in the Piano Roll, arrange patterns on the Playlist and mix them on
Mixer inserts. The audio engine, the project model and the native [egui](https://github.com/emilk/egui) interface are
written in Rust. The app runs locally without a web view, account or network connection.

## Make a Beat, Start to Finish

The core composition loop has automated coverage in
[`mvp_workflow.rs`](crates/limeshot-app/tests/mvp_workflow.rs). Click a preview to see it at full size.

| Feature | Preview |
| --- | --- |
| **Channel Rack**<br>Program drums from samples on a step grid, with swing, fill, rotate and channel selection. | <a href="docs/screenshots/current/rack.png"><img src="docs/screenshots/current/rack.png" width="240" alt="Channel Rack and Playlist"></a> |
| **Piano Roll**<br>Write notes for LimeSynth, the Sampler or a VST3 instrument. It has FL's eight tools, ghost notes, and velocity and pan lanes. | <a href="docs/screenshots/current/piano-roll.png"><img src="docs/screenshots/current/piano-roll.png" width="240" alt="Piano Roll"></a> |
| **Playlist**<br>Arrange pattern, audio and automation clips. Slip, slice, audio clip fades, time selection, markers and a clip picker. | <a href="docs/screenshots/current/audio-clips.png"><img src="docs/screenshots/current/audio-clips.png" width="240" alt="Playlist audio waveforms, clip fades, markers and time selection"></a> |
| **Mixer**<br>Route channels to inserts with volume, pan, a dedicated hosted EQ, sidechain compression and up to ten effects per track. Each effect slot has On and a Mix knob, and you reorder slots by dragging. | <a href="docs/screenshots/current/mixer.png"><img src="docs/screenshots/current/mixer.png" width="240" alt="Mixer"></a> |
| **Workspace windows**<br>Move, resize, maximise and detach editors; F5/F6/F7/F9 toggle Playlist, Rack, Piano Roll and Mixer. | <a href="docs/screenshots/current/workspace.png"><img src="docs/screenshots/current/workspace.png" width="240" alt="Current workspace with floating Channel Rack"></a> |
| **Automation**<br>Shape Song-mode pan automation with curves, stairs and waves. | <a href="docs/screenshots/current/automation.png"><img src="docs/screenshots/current/automation.png" width="240" alt="Playlist pan automation with curve, stairs and wave segments"></a> |
| **Sampler**<br>Inspect the sample waveform and adjust playback, loop and filter settings. | <a href="docs/screenshots/current/sampler.png"><img src="docs/screenshots/current/sampler.png" width="240" alt="Sampler Channel Settings with waveform and playback controls"></a> |
| **LimeSynth**<br>Edit oscillators, envelopes, filtering and modulation. | <a href="docs/screenshots/current/synth.png"><img src="docs/screenshots/current/synth.png" width="240" alt="Built-in LimeSynth Channel Settings"></a> |
| **Mixer EQ**<br>Shape the selected insert with the inline three-band hosted EQ. | <a href="docs/screenshots/current/mixer-eq.png"><img src="docs/screenshots/current/mixer-eq.png" width="240" alt="Maximised Mixer with the selected insert EQ"></a> |
| **WAV export**<br>Choose the render range, sample rate and bit depth. | <a href="docs/screenshots/current/export.png"><img src="docs/screenshots/current/export.png" width="240" alt="WAV export dialog over the audio clip arrangement"></a> |

## Sounds, Effects and Plugins

| Kind | What's included |
| --- | --- |
| **Instruments** | LimeSynth (PolyBLEP oscillators, ADSR, low-/band-/high-pass filter, LFO) and Lime Sampler (ADSR, trim, reverse, loop points, low-pass filter; samples up to about 125 s at 48 kHz), both hosted as Lime Native VST3s; bundled Lime samples and factory presets |
| **Effects** | Lime EQ, Filter, tempo-synced Delay, Room (reverb), Comp (with sidechain), Limiter, Gate, Drive, Chorus (chorus/flanger/phaser) and Utility |
| **Plugins** | VST3 instruments and effects: scanning into a plugin database, state saved in the project, and an internal Plugin / Settings window (macOS native embedding awaits live verification) |
| **Automation** | Song-mode clips for channel/mixer volume and pan, hosted VST3 parameters; point modes and visible automated controls |
| **Recording** | MIDI input and note recording from a device or typing keyboard; audio input recording to Playlist clips is implemented but awaits real-input verification |
| **Files** | `.lime` projects (versioned, atomic save, autosave/recovery), WAV/MP3/FLAC/Ogg Vorbis/AIFF samples, WAV export of a song/pattern/time selection, MIDI import/export and partial FL Studio `.flp` import |

## Project Status

**Early alpha.** LimeShot can compose, arrange, mix, save and render songs, but production readiness and platform
coverage are still being established. Automated tests establish the behavior they exercise; they do not establish
compatibility with every audio device, plugin or host. README screenshots were freshly rendered on 2026-10-23
from the source commits recorded in
[the screenshot notes](docs/screenshots/README.md). They show production UI in offscreen demo scenarios; live native
plugin embedding and device audio require separate checks. The Mixer screenshots predate the 2026-10-25 insert-slot
redesign (On + Mix, drag handle).

The inventory currently tracks **291 features: 144 VERIFIED, 50 IMPLEMENTED, 78 NOT_STARTED, 14 IN_PROGRESS, 4 DEFERRED, 1 RESEARCHED** (2026-10-25). IMPLEMENTED means code exists; VERIFIED additionally
requires listed tests or recorded manual evidence. These counts describe tracked features, not percentage parity
with FL Studio or measured test coverage.

| State tracker | What it records |
| --- | --- |
| [Feature inventory](docs/roadmap/features.md) / [canonical JSON](docs/roadmap/features.json) | Priorities, dependencies, acceptance criteria, implementation states and verification evidence |
| [Menu inventory](docs/roadmap/menus.md) | 737 menu/toolbar/editor entries and their implementation differences |
| [Shortcut inventory](docs/roadmap/keybindings.md) | 143 bindings, including differences and unimplemented shortcuts |
| [Settings inventory](docs/roadmap/settings.md) | 189 controls/settings and their current states |
| [MVP acceptance](docs/roadmap/mvp.md) / [milestones](docs/roadmap/milestones.md) | End-to-end acceptance and delivery scope |
| [Progress log](docs/development/progress.md) / [next steps](docs/development/next-steps.md) | Dated verification records and continuation priorities |
| [Known limitations](docs/roadmap/known-limitations.md) | Outstanding restrictions and manual/platform checks |
| [Live CI](https://github.com/fruitapps/limeshot/actions/workflows/ci.yml) | Build, test and lint results for the actual commit and platform |

The DAW now uses the twelve Lime Native VST3 bundles for its instruments and effects. Build them with
`python3 native-plugins/build.py` before running from source; macOS packages include them. Legacy projects convert
to portable `lime:` references with embedded sampler audio. Hosted Playlist clips, note pan, exact track EQ, all ten effect kinds and mixer key buses are implemented; see [known limitations](docs/roadmap/known-limitations.md).

Some older continuation/limitation entries predate the recording, decoder, sampler, fade and automation additions;
use the canonical feature record and its latest progress evidence to resolve those historical descriptions.

Still missing or limited: CLAP/AU, stems, templates, remappable shortcuts, automatic workspace
layout persistence, time stretching and several advanced editor actions. Hardware MIDI and audio recording need
hands-on checks; audio recording has no input monitoring or latency compensation. Native macOS VST3 embedding
needs another live resize/tab/keyboard pass; Windows embeds editors as child windows (untested on real hardware); Linux exposes plugin parameters. The process bridge
is opt-in and has no native editor. `.flp` import omits plugin state, mixer effects/routing and automation curves;
unsupported generators become LimeSynth. Integer WAV export has no dither; Gate/Drive/Chorus use their standalone Lime VST3 replacements.

## Lime Native Standalone VST3 Suite

**Lime Native** packages two instruments and ten effects as twelve independent VST3 plugins. Each has its own
resizable native editor, factory presets, host automation/state and `.limepreset` interchange, with the existing
LimeShot DSP linked into the bundle. LimeShot does not need to be installed or running.

Previews of all twelve standalone editors (click to enlarge):

| | | |
| --- | --- | --- |
| <a href="docs/screenshots/current/native/limesynth.png"><img src="docs/screenshots/current/native/limesynth.png" width="280" alt="LimeSynth standalone VST3 editor"></a><br>**LimeSynth** | <a href="docs/screenshots/current/native/lime-sampler.png"><img src="docs/screenshots/current/native/lime-sampler.png" width="280" alt="Lime Sampler standalone VST3 editor"></a><br>**Lime Sampler** | <a href="docs/screenshots/current/native/lime-utility.png"><img src="docs/screenshots/current/native/lime-utility.png" width="280" alt="Lime Utility standalone VST3 editor"></a><br>**Lime Utility** |
| <a href="docs/screenshots/current/native/lime-eq.png"><img src="docs/screenshots/current/native/lime-eq.png" width="280" alt="Lime EQ standalone VST3 editor"></a><br>**Lime EQ** | <a href="docs/screenshots/current/native/lime-filter.png"><img src="docs/screenshots/current/native/lime-filter.png" width="280" alt="Lime Filter standalone VST3 editor"></a><br>**Lime Filter** | <a href="docs/screenshots/current/native/lime-delay.png"><img src="docs/screenshots/current/native/lime-delay.png" width="280" alt="Lime Delay standalone VST3 editor"></a><br>**Lime Delay** |
| <a href="docs/screenshots/current/native/lime-room.png"><img src="docs/screenshots/current/native/lime-room.png" width="280" alt="Lime Room standalone VST3 editor"></a><br>**Lime Room** | <a href="docs/screenshots/current/native/lime-comp.png"><img src="docs/screenshots/current/native/lime-comp.png" width="280" alt="Lime Comp standalone VST3 editor"></a><br>**Lime Comp** | <a href="docs/screenshots/current/native/lime-limiter.png"><img src="docs/screenshots/current/native/lime-limiter.png" width="280" alt="Lime Limiter standalone VST3 editor"></a><br>**Lime Limiter** |
| <a href="docs/screenshots/current/native/lime-gate.png"><img src="docs/screenshots/current/native/lime-gate.png" width="280" alt="Lime Gate standalone VST3 editor"></a><br>**Lime Gate** | <a href="docs/screenshots/current/native/lime-drive.png"><img src="docs/screenshots/current/native/lime-drive.png" width="280" alt="Lime Drive standalone VST3 editor"></a><br>**Lime Drive** | <a href="docs/screenshots/current/native/lime-chorus.png"><img src="docs/screenshots/current/native/lime-chorus.png" width="280" alt="Lime Chorus standalone VST3 editor"></a><br>**Lime Chorus** |

| Plugin | Purpose |
| --- | --- |
| **LimeSynth** | Polyphonic oscillator synth with ADSR, resonant low-/band-/high-pass filter, key tracking and LFO |
| **Lime Sampler** | Stereo sampler (WAV, MP3, FLAC, Ogg Vorbis, AIFF) with trim/reverse, root key, audition keyboard and embedded sample recall |
| **Lime Utility** | Gain, stereo width and polarity |
| **Lime EQ** | Three-band EQ with draggable response nodes |
| **Lime Filter** | Resonant low-, band- and high-pass filtering |
| **Lime Delay** | Stereo delay with host-tempo sync |
| **Lime Room** | Algorithmic stereo reverb |
| **Lime Comp** | Compressor with optional external stereo sidechain |
| **Lime Limiter** | Sample-peak limiter with gain-reduction history; no lookahead or true-peak oversampling |
| **Lime Gate** | Noise gate with hold and adjustable range |
| **Lime Drive** | Soft/hard clipping and wave folding with tone and mix |
| **Lime Chorus** | Chorus, flanger and phaser modes |

The suite is included in this checkout under `native-plugins/`. Its
[README](native-plugins/README.md),
[validation record](native-plugins/VALIDATION.md)
and [third-party notices](native-plugins/THIRD_PARTY.md)
cover building, installing, presets and supported targets. To build the suite (Rust **1.95+**, Python 3
and platform build tools):

```bash
python3 native-plugins/build.py
```

The script collects bundles and a ZIP in `native-plugins/dist/`; it does not install them. Copy the `.vst3` bundles
to your host's plugin directory and rescan. Current macOS Apple Silicon release/debug builds pass exported ABI checks for all twelve bundles. The historical
nine-bundle baseline passed pluginval 1.0.4 strictness 5; pluginval is unavailable for the current binaries. Hands-on DAW checks, Windows/Linux/Intel runtime
validation and notarization remain pending. Local macOS bundles are arm64 and ad-hoc signed. Generated binaries
are build outputs, not files included by cloning Git. VST3-compatible hosts are required; Logic Pro needs AU.
The suite exports twelve plugins, including Lime Gate, Lime Drive and Lime Chorus.

## Platforms

| Platform | State |
| --- | --- |
| macOS Apple Silicon | Built, all tests pass, GUI, realtime audio and VST3 processing tested; new native editor embedding unverified |
| macOS Intel | Native headless CI passes; earlier Rosetta render matched arm64; GUI/audio-device checks remain open |
| Windows x86-64 / ARM64 | x86-64 headless CI and ARM64 cross-build pass; native GUI/audio checks remain open |
| Linux x86-64 / ARM64 | Native headless CI passes; GUI/audio-device checks remain open |

Commit-specific CI evidence is recorded in [platform-validation.md](docs/development/platform-validation.md);
successful results apply to the listed commits, and later commits need their own CI runs.

## Build from Source

The DAW manifest declares Rust 1.88 or newer; current development/CI uses stable Rust.
The separate Lime Native workspace requires Rust 1.95 or newer. On Linux, install the ALSA and windowing headers first:

```bash
sudo apt install libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev \
  libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libgtk-3-dev mesa-vulkan-drivers
```

Run the app, or open the bundled demo song:

```bash
git clone https://github.com/fruitapps/limeshot.git
cd limeshot
cargo run --release -p limeshot-app
cargo run --release -p limeshot-app -- --demo
```

Press **Space** to play, **L** to switch between Pattern and Song mode, **F5/F6/F7/F9** to toggle the Playlist/Rack/Piano Roll/Mixer and **F1** to see
every shortcut. Enable typing-keyboard notes in the toolbar to play the selected Channel Rack instrument;
Cmd+T toggles this outside the Playlist (where it adds a marker).

On macOS, `bash scripts/package-macos.sh` builds an app and DMG in `dist/`. These are ad-hoc signed development
packages; Developer ID signing and notarization are pending. Windows/Linux installers are not yet provided.

<details>
<summary><strong>Headless CLI</strong></summary>

```bash
mkdir -p out
cargo run --release -p limeshot-app --bin limeshot-cli -- demo out/demo.lime --render out/demo.wav
cargo run --release -p limeshot-app --bin limeshot-cli -- import-flp song.flp out/song.lime
cargo run --release -p limeshot-app --bin limeshot-cli -- midi out/demo.lime out/demo.mid
```

`limeshot-cli` also has `info`, `render`, `play` (realtime through the audio device), `import-midi` and `run`. `run`
applies a JSON list of the same typed commands the UI uses. Opening a legacy project does not automatically embed
audio outside its folder. Select the source explicitly in the GUI, or use `--allow-external-samples` in the CLI
when you intend to include those recordings in portable state:


```json
[{"SetTempo": 128000}, {"AddPattern": {"name": "Verse"}}]
```

</details>

<details>
<summary><strong>Tests</strong></summary>

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/features.py check
python3 scripts/check_repository.py
```

The suite covers musical timing, scheduling across buffer sizes and sample rates, synth and sampler behaviour,
effects, mixing, VST3 hosting, project round trips, undo and redo, and the editors driven with synthetic input.

</details>

## Architecture

| Crate | What it does |
| --- | --- |
| `limeshot-core` | Musical time (960 PPQ ticks), the project model, typed commands, undo and redo |
| `limeshot-project` | `.lime` files (versioned envelope, migrations, atomic saving), MIDI, and `.flp` import |
| `limeshot-dsp` | Oscillators, envelopes, effect processors, pan law and audio decoding/WAV writing |
| `limeshot-sequencer` | Compiles a project into an immutable playback snapshot |
| `limeshot-engine` | The renderer (voices, scheduling, mixer, transport, effects), shared by playback and export |
| `limeshot-audio` | cpal device output/input recording, MIDI input, lock-free control and background sample loading |
| `limeshot-plugins` | VST3 hosting: loading, scanning, processing, state and editor windows |
| `limeshot-ui` | The egui interface: toolbar, browser, Channel Rack, Piano Roll, Playlist, Mixer, windows |
| `limeshot-app` | The `limeshot` desktop app and `limeshot-cli` |

For contributor setup, regression tests, real-time constraints, plugin compatibility and pull requests, read
[CONTRIBUTING.md](CONTRIBUTING.md). Agent-specific guidance remains in [AGENTS.md](AGENTS.md).

## Documentation

- [Contributing](CONTRIBUTING.md)
- [Architecture](docs/architecture/): audio engine, project model and UI
- [Design decisions](docs/development/decisions.md)
- [Roadmap and feature inventory](docs/roadmap/)
- [Research](docs/research/): how the reference application was studied
- [Working on LimeShot with agents](AGENTS.md)

## License

LimeShot is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Bundled assets are
listed in [ATTRIBUTION.md](ATTRIBUTION.md).

LimeShot is independent and not affiliated with Image-Line. FL Studio is a trademark of Image-Line. No FL Studio code,
artwork, samples or presets are used. The reference application was studied only through its documentation, metadata
and observable behaviour (see [docs/research/](docs/research/)).

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=fruitapps/limeshot&type=Date&legend=top-left)](https://www.star-history.com/?repos=fruitapps%2Flimeshot&type=date&legend=top-left)

GitHub badges and the star-history chart are external README services and become useful once the repository is
public. They are separate from the offline application.
