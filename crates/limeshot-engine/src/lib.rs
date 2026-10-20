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
pub mod renderer;
pub mod voice;

pub use export::{ExportError, ExportRange, RenderReport, RenderSettings, export_wav, render_offline};
pub use limeshot_dsp::BitDepth;
pub use renderer::{MAX_BLOCK, MAX_VOICES, Peak, PlayMode, Renderer};
