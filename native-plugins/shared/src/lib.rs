//! Lime Native: the same sound engines as LimeShot, packaged for other DAWs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
pub mod editor;
pub mod parameters;
mod processor;
pub mod sample;
pub use processor::LimePlugin;

mod effects;
