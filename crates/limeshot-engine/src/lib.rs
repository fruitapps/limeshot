//! The LimeShot engine: transport, scheduling, voices and mixing ([`Renderer`]), and offline
//! export ([`export`]). It has no device or UI code; `limeshot-audio` drives it from a cpal
//! callback and the CLI/UI drive it offline.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod export;
pub mod fx;
mod pdc;
mod plugins;
pub mod renderer;

pub use export::{
    ExportError, ExportFormat, ExportRange, MP3_BITRATES, MP3_SAMPLE_RATES, Processors, RenderReport, RenderSettings,
    StemReport, export_audio_with, export_stems_with, export_wav, export_wav_with, render_offline, render_offline_with,
    stem_path,
};
pub use limeshot_dsp::BitDepth;
pub use plugins::MAX_PLUGIN_CHANNELS;
pub use renderer::{MAX_BLOCK, MAX_VOICES, Peak, PlayMode, Renderer};
