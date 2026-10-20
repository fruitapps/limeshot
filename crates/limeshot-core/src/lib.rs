//! LimeShot core: musical time, the project document, and the typed commands that edit it.
//!
//! This crate has no audio or UI dependencies. Everything above it (sequencer, engine, UI, CLI)
//! reads a [`Project`] and changes it only through [`Command`]s executed on a [`Document`].
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod command;
pub mod document;
pub mod model;
pub mod time;

pub use command::{Command, CommandError, Created, NewNote};
pub use document::Document;
pub use model::*;
pub use time::{BarBeatTick, PPQ, STEP_TICKS, Snap, Tempo, Tick, TimeSignature, Timeline};
