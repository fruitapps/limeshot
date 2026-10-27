# LimeShot patch to nice-plug 0.4.2

Upstream crates.io source, ISC license retained. In VST3 setupProcessing, construct BufferConfig after decoding the requested processing mode. Previously it retained the previous mode, making first-time offline activation appear realtime to the plugin. Additional review fixes are listed below. Hosted disk sampler seek tests cover offline waiting across four block sizes.

Source provenance: crates.io nice-plug 0.4.2, upstream commit `91ba75d20c43ca0cd21afa802d0ae8c0511f48ed`, `crates/nice-plug`. The published crate omits the workspace license file; LICENSE is retained from that exact upstream commit.

## PR review follow-up

- Background worker skips stale executors and tolerates disconnected shutdown/join errors.
- Buffer tails clamp host channel counts to the configured slice count.
- VST3 validates parameter indices and aux bus boundaries/offsets, including layouts without main outputs.
- Expression and Brightness use their matching note-event variants.
- Suggested DPI scale is validated and retained before an editor opens.
- README wgpu links use absolute URLs so repository reference checks work.
- Windows hidden event-loop window classes use a DLL-local atomic sequence plus the sequence's address.
  Performance-counter ticks can collide during concurrent plugin creation and abort the host at
  `RegisterClassExA`; the address also separates framework copies in different plugin DLLs.
  `concurrent_event_loops_register_distinct_window_classes` creates/destroys 512 windows across eight
  threads and checks GUI task dispatch and unique class names. Native-plugin CI runs the vendor unit tests.

The normalized Cargo manifest declares an independent workspace so its regression tests can run directly:
`CARGO_TARGET_DIR="$PWD/native-plugins/target" cargo test --manifest-path native-plugins/vendor/nice-plug/Cargo.toml --no-default-features --features vst3 --lib`.
