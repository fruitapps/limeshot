# LimeShot patch to nice-plug 0.4.2

Upstream crates.io source, ISC license retained. In VST3 setupProcessing, construct BufferConfig after decoding the requested processing mode. Previously it retained the previous mode, making first-time offline activation appear realtime to the plugin. No other source changes. Hosted disk sampler seek tests cover offline waiting across four block sizes.
