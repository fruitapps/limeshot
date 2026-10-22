# Lime Native dependencies

Lime Native source is MIT OR Apache-2.0, like LimeShot. The plugins statically link LimeShot's
original core, DSP, sequencer, engine and preset code; no LimeShot installation is required.

The pinned dependency graph is recorded in `Cargo.lock`. Key runtime dependencies:

| Dependency | License | Purpose |
| --- | --- | --- |
| nice-plug 0.4.2 / nice-plug-core | ISC | Plugin lifecycle, parameter/state handling, VST3 export |
| vst3 0.3 | MIT OR Apache-2.0 | VST3 ABI bindings generated from the MIT-licensed SDK |
| nice-plug-egui 0.6 / egui-baseview / baseview | ISC / MIT OR Apache-2.0 / MIT | Embedded native editor |
| egui 0.37 | MIT OR Apache-2.0 | Interface rendering |
| rfd | MIT | Native file dialogs |
| serde / serde_json / base64 | MIT OR Apache-2.0 | Parameter/preset/sample state |
| hound | Apache-2.0 | WAV reading/writing |
| symphonia 0.5 | MPL-2.0 | Decoder dependency of the shared DAW DSP crate; upstream files are unmodified |
| parking_lot | MIT OR Apache-2.0 | State loading outside realtime processing |

Editors reuse LimeShot's bundled Inter and JetBrains Mono fonts. Their licenses and attribution
are in `../ATTRIBUTION.md` and `../assets/fonts/`; build.py also copies license texts into `dist/licenses/`.
No additional graphic assets are used.
VST is a trademark of Steinberg Media Technologies GmbH. LimeShot is an independent implementation.
