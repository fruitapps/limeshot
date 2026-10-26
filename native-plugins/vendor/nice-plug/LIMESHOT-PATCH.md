# LimeShot patch to nice-plug 0.4.2

Upstream crates.io source, ISC license retained. In VST3 setupProcessing, construct BufferConfig after decoding the requested processing mode. Previously it retained the previous mode, making first-time offline activation appear realtime to the plugin. No other source changes. Hosted disk sampler seek tests cover offline waiting across four block sizes.

Source provenance: crates.io nice-plug 0.4.2, upstream commit `91ba75d20c43ca0cd21afa802d0ae8c0511f48ed`, `crates/nice-plug`. The published crate omits the workspace license file; LICENSE is retained from that exact upstream commit.
