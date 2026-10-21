<h1 align="center">LimeShot</h1>

<p align="center"><strong>The pattern-based DAW, in pure Rust.</strong></p>

<p align="center">
  <a href="docs/screenshots/live-macos.png"><img src="docs/screenshots/live-macos.png" alt="LimeShot on macOS: Channel Rack, Piano Roll, Playlist and Mixer" width="900"></a>
</p>

<p align="center">
  <a href="#build-from-source">Build from source</a> ·
  <a href="docs/roadmap/features.md">Feature inventory</a> ·
  <a href="docs/roadmap/known-limitations.md">Known limitations</a> ·
  <a href="AGENTS.md">Contributing</a>
</p>

LimeShot is an open-source, offline-first digital audio workstation built around the pattern workflow of FL Studio:
program beats in the Channel Rack, write melodies in the Piano Roll, arrange patterns on the Playlist and mix them on
Mixer inserts. The audio engine, the project model and the native [egui](https://github.com/emilk/egui) interface are
all Rust. There is no web view, no account and no network.

## Make a Beat, Start to Finish

Every workflow below is covered by automated tests, and the core loop runs end to end in
[`mvp_workflow.rs`](crates/limeshot-app/tests/mvp_workflow.rs). Click a preview to see it at full size.

| Feature | Preview |
| --- | --- |
| **Channel Rack**<br>Program drums from WAV samples on a step grid, with swing, fill and rotate. | <a href="docs/screenshots/main.png"><img src="docs/screenshots/main.png" width="240" alt="Channel Rack and Playlist"></a> |
| **Piano Roll**<br>Write notes for LimeSynth, the Sampler or a VST3 instrument. It has FL's eight tools, ghost notes, and velocity and pan lanes. | <a href="docs/screenshots/piano-roll.png"><img src="docs/screenshots/piano-roll.png" width="240" alt="Piano Roll"></a> |
| **Playlist**<br>Arrange pattern, audio and automation clips. Slip, slice, time selection, markers and a clip picker. | <a href="docs/screenshots/live-song-mixer.png"><img src="docs/screenshots/live-song-mixer.png" width="240" alt="Playlist in Song mode with the Mixer"></a> |
| **Mixer**<br>Route channels to inserts with volume, pan, a built-in EQ, sidechain compression and up to ten effects per track. | <a href="docs/screenshots/mixer.png"><img src="docs/screenshots/mixer.png" width="240" alt="Mixer"></a> |
| **Workspace windows**<br>Maximise, detach and toggle every editor with F5–F9, as in FL Studio. | <a href="docs/screenshots/live-mixer-detached.png"><img src="docs/screenshots/live-mixer-detached.png" width="240" alt="Detached Mixer window"></a> |

## Sounds, Effects and Plugins

| Kind | What's included |
| --- | --- |
| **Instruments** | LimeSynth (PolyBLEP oscillators, ADSR, low-pass filter, LFO) and a WAV Sampler, with bundled Lime samples |
| **Effects** | Lime EQ, Filter, Delay, Room (reverb), Comp (with sidechain), Limiter and Utility |
| **Plugins** | VST3 instruments and effects: scanning into a plugin database, state saved in the project, and the plugin's own editor window on macOS |
| **Automation** | Automation clips for channel and mixer volume and pan and effect parameters, in Song mode |
| **Files** | `.lime` projects (versioned, atomic save), WAV export, MIDI import and export, and FL Studio `.flp` import |

<details>
<summary><strong>Not there yet</strong></summary>

Recording and audio input, MIDI devices, tempo and plugin-parameter automation, plugin delay compensation, CLAP and
Audio Units, MP3/FLAC/OGG, stems and templates. The full list is in
[known-limitations.md](docs/roadmap/known-limitations.md). The inventory tracks 239 features in
[features.md](docs/roadmap/features.md), and 130 of them are verified by tests.

</details>

## Platforms

| Platform | State |
| --- | --- |
| macOS Apple Silicon | Built, all tests pass, GUI, realtime audio and VST3 plugins verified |
| macOS Intel | Release build runs under Rosetta; headless render is bit-identical to arm64; GUI not exercised |
| Windows x86-64 / ARM64 | Type-checks; CI jobs defined, not yet run |
| Linux x86-64 / ARM64 | CI jobs defined, not yet run |

## Build from Source

Requires stable Rust 1.88 or newer. On Linux, install the ALSA and windowing headers first:

```bash
sudo apt install libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev libgtk-3-dev
```

Run the app, or open the bundled demo song:

```bash
cargo run --release -p limeshot-app
cargo run --release -p limeshot-app -- --demo
```

Press **Space** to play, **L** to switch between Pattern and Song mode, **F5–F9** to toggle windows and **F1** to see
every shortcut.

<details>
<summary><strong>Headless CLI</strong></summary>

```bash
cargo run --release -p limeshot-app --bin limeshot-cli -- demo out/demo.lime --render out/demo.wav
cargo run --release -p limeshot-app --bin limeshot-cli -- import-flp song.flp out/song.lime
cargo run --release -p limeshot-app --bin limeshot-cli -- midi out/demo.lime out/demo.mid
```

`limeshot-cli` also has `info`, `render`, `play` (realtime through the audio device), `import-midi` and `run`. `run`
applies a JSON list of the same typed commands the UI uses:

```json
[{"SetTempo": 128000}, {"AddPattern": {"name": "Verse"}}]
```

</details>

<details>
<summary><strong>Tests</strong></summary>

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The suite covers musical timing, scheduling across buffer sizes and sample rates, synth and sampler behaviour,
effects, mixing, VST3 hosting, project round trips, undo and redo, and the editors driven with synthetic input.

</details>

## Architecture

| Crate | What it does |
| --- | --- |
| `limeshot-core` | Musical time (960 PPQ ticks), the project model, typed commands, undo and redo |
| `limeshot-project` | `.lime` files (versioned envelope, migrations, atomic saving), MIDI, and `.flp` import |
| `limeshot-dsp` | Oscillators, envelopes, pan law, WAV I/O |
| `limeshot-sequencer` | Compiles a project into an immutable playback snapshot |
| `limeshot-engine` | The renderer (voices, scheduling, mixer, transport, effects), shared by playback and export |
| `limeshot-audio` | cpal device output, lock-free control and background sample loading |
| `limeshot-plugins` | VST3 hosting: loading, scanning, processing, state and editor windows |
| `limeshot-ui` | The egui interface: toolbar, browser, Channel Rack, Piano Roll, Playlist, Mixer, windows |
| `limeshot-app` | The `limeshot` desktop app and `limeshot-cli` |

## Documentation

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
