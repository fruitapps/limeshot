# Bundled assets

Every file LimeShot ships besides source code is listed here.

| Path | Author | Source | License |
|---|---|---|---|
| `assets/samples/Lime *.wav` (6 drum sounds) | LimeShot contributors | Synthesised by `crates/limeshot-dsp/examples/make_samples.rs` (oscillators + seeded noise); regenerate with `cargo run -p limeshot-dsp --example make_samples` | MIT OR Apache-2.0 |
| App icon, logo, all UI icons | LimeShot contributors | Drawn in code: `crates/limeshot-ui/src/icons.rs`, `crates/limeshot-app/src/lib.rs` (`icon_rgba`) | MIT OR Apache-2.0 |
| `docs/screenshots/*.png` | LimeShot contributors | Offscreen renders of LimeShot (`ui_shot` example) | MIT OR Apache-2.0 |
| UI fonts (Ubuntu-Light, Hack, emoji) | Canonical, Source Foundry, Google/Twitter | Embedded by egui's `default_fonts` | Ubuntu Font Licence 1.0, MIT, OFL-1.1 / Apache-2.0 |

No FL Studio, Image-Line, or ArtCraft artwork, samples, presets, fonts or code are included.

## Dependency licences

`cargo metadata` (2026-10-20): all Rust dependencies are permissive (MIT, Apache-2.0, BSD, ISC, Zlib, BSL-1.0, Unicode, Unlicense)
or offer a permissive alternative, except two unmodified MPL-2.0 crates (`option-ext`, via `directories`; `colored`, test tooling),
whose file-level copyleft does not extend to LimeShot's code.
