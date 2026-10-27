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

pub mod automation;
pub mod command;
pub mod document;
pub mod effects;
pub mod lime;
pub mod model;
pub mod note_tools;
pub mod plugin;
pub mod time;

pub use automation::{Automation, AutomationId, AutomationPoint, AutomationTarget, Mapping, PointMode};
pub use command::{Command, CommandError, Created, ImportedChannel, NewNote, NoteProp};
pub use document::{Document, PluginTarget};
pub use effects::{COMPRESSOR_SIDECHAIN, DELAY_SYNC, EffectKind, EffectSlot, MAX_EFFECTS, ParamSpec};
pub use lime::{EmbeddedAudio, LimePlugin};
pub use model::*;
pub use note_tools::NoteTool;
pub use plugin::{PluginRef, PluginState, PluginWindow};
pub use time::{BarBeatTick, MAX_TICK, PPQ, STEP_TICKS, Snap, Tempo, Tick, TimeSignature, Timeline};
