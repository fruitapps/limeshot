# Bundled assets

Every file LimeShot ships besides source code is listed here.

| Path | Author | Source | License |
|---|---|---|---|
| `assets/samples/Lime *.wav` (6 drum sounds) | LimeShot contributors | Synthesised by `crates/limeshot-dsp/examples/make_samples.rs` (oscillators + seeded noise); regenerate with `cargo run -p limeshot-dsp --example make_samples` | MIT OR Apache-2.0 |
| `crates/limeshot-dsp/tests/fixtures/sine.aiff`, `sine.flac` (test fixtures, not shipped) | LimeShot contributors | A 0.1 s sine written by a script, converted with `afconvert` (AIFF) and a hand-written verbatim FLAC encoder (session 16) | MIT OR Apache-2.0 |
| App icon, logo, all UI icons | LimeShot contributors | Drawn in code: `crates/limeshot-ui/src/icons.rs`, `crates/limeshot-app/src/lib.rs` (`icon_rgba`) | MIT OR Apache-2.0 |
| `docs/screenshots/*.png` | LimeShot contributors | Offscreen renders (`ui_shot`) and live macOS captures of LimeShot | MIT OR Apache-2.0 |
| `assets/fonts/Inter-Regular.ttf`, `Inter-SemiBold.ttf` | The Inter Project Authors (Rasmus Andersson) | https://github.com/rsms/inter/releases/tag/v4.1 — licence text in `assets/fonts/OFL-Inter.txt` | SIL OFL 1.1 |
| `assets/fonts/JetBrainsMono-Regular.ttf` | The JetBrains Mono Project Authors | https://github.com/JetBrains/JetBrainsMono/releases/tag/v2.304 — licence text in `assets/fonts/OFL-JetBrainsMono.txt` | SIL OFL 1.1 |
| Fallback fonts (Ubuntu-Light, Hack, Noto Emoji, emoji-icon-font) | Canonical, Source Foundry / Bitstream / DejaVu, Google, John Slegers | Embedded by `epaint_default_fonts` 0.36.2; upstream licence texts copied to `assets/fonts/{UFL-Ubuntu,OFL-NotoEmoji,LICENSE-Hack,LICENSE-emoji-icon-font}.txt` | Ubuntu Font Licence 1.0, MIT / Bitstream Vera, SIL OFL 1.1, MIT |

No FL Studio, Image-Line, or ArtCraft artwork, samples, presets, fonts or code are included.

## Dependency licences

The resolved Apple Silicon dependency metadata was checked on 2026-10-22. Dependencies offer permissive
licences (including MIT, Apache-2.0, BSD, ISC, Zlib, BSL-1.0, Unicode and Unlicense), except the unmodified MPL-2.0
crates `option-ext`, `colored` (test tooling) and the `symphonia` decoder family. The fallback font crate also
includes SIL OFL 1.1 and Ubuntu Font Licence 1.0 assets, with the font notices listed above.

VST3 hosting (2026-10-21) adds `vst3` 0.3 and `com-scrape-types` (MIT OR Apache-2.0; bindings generated from the VST 3.8 SDK
headers, which Steinberg publishes under the MIT licence) and `libloading` (ISC). No SDK source is included. "VST" is a
trademark of Steinberg Media Technologies GmbH.

Sample decoding (2026-10-22) adds `symphonia` 0.5 (MPL-2.0, unmodified, from crates.io) for MP3, FLAC, Ogg Vorbis and AIFF;
`midir` (MIT) reads MIDI input.


The macOS bundle includes this file, LimeShot's licence texts and all font licence texts in
`Contents/Resources/licenses/fonts`. Aggregation of third-party Rust source licence notices into distribution
packages remains open; the dependency overview above is not a complete set of redistribution notices.

MP3 export (2026-10-27) adds unmodified `rusty_mp3` **1.0.5** from crates.io, Apache-2.0,
Copyright 2026 Mata Network (https://github.com/Remade-With-Rust/remade_ffmpeg_rs).
Default features are disabled; it is pure Rust with no runtime dependencies. Its Apache-2.0
license text is the same license shipped in `LICENSE-APACHE`; the package contains no NOTICE file.
No FL Studio or LAME encoder binary is bundled.

## Vendored editor backend

| Source | Authors | License | Use |
|---|---|---|---|
| `native-plugins/vendor/egui-baseview` | egui-baseview 0.9.0, RustAudio contributors | MIT OR Apache-2.0; license texts retained | Locally patched renderer teardown; see `LIMESHOT-PATCH.md` in that directory |

| `native-plugins/vendor/nice-plug` | nice-plug 0.4.2 contributors | ISC; license retained | VST3 processing-mode setup fix; see `LIMESHOT-PATCH.md` |
