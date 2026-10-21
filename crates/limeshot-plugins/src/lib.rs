//! LimeShot plugin hosting: VST3 instruments and effects.
//!
//! This is the only crate in the workspace that may use `unsafe` (decision D27): the VST3 API is a
//! C++ COM ABI, reached through the generated `vst3` bindings. Everything unsafe stays behind the
//! safe types exported here:
//!
//! * [`Vst3Module`] loads a `.vst3` bundle (or a factory compiled into the program) and lists its
//!   classes as [`PluginInfo`].
//! * [`scan`] finds bundles in the platform's standard folders.
//! * [`Vst3Instance::new`] creates a plugin and splits it into the control-thread side
//!   ([`Vst3Instance`]: parameters, state) and the audio-thread side ([`Vst3Processor`]: audio and
//!   notes), which implements [`limeshot_dsp::effects::Effect`].
//! * [`Vst3Instance::create_editor`] opens the plugin's own editor ([`Vst3Editor`]) for a window
//!   of the host's ([`NativeWindow`]).
//! * [`testing`] holds small plugins of LimeShot's own for tests that must not depend on what is
//!   installed.
//!
//! Threading follows the VST3 rules: creation, parameters, state and teardown on the control
//! thread; `process` on the audio thread. [`Vst3Processor::process`] does not allocate or lock.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
// The VST3 enums are `u32` on macOS and Linux and `i32` on Windows, so casts that one platform
// needs are redundant on another.
#![allow(clippy::unnecessary_cast)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod editor;
mod host;
mod instance;
mod module;
pub mod preset;
pub mod scan;
pub mod testing;
mod util;

pub use editor::{EditorSize, NativeWindow, Vst3Editor};
pub use instance::{NoteEvent, NoteEventKind, ParamInfo, Transport, Vst3Instance, Vst3Processor};
pub use module::Vst3Module;
pub use preset::VstPreset;

use std::path::PathBuf;

/// What a plugin class is used as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginKind {
    /// Makes sound from notes (a Channel Rack channel).
    Instrument,
    /// Processes audio (a mixer insert slot).
    Effect,
}

/// One plugin class found in a bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginInfo {
    /// The bundle the class lives in (empty for a factory compiled into the program).
    pub path: PathBuf,
    /// The class id as 32 hexadecimal digits; with `path` it identifies the plugin.
    pub class_id: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    /// VST3 sub-categories, such as `Instrument|Synth` or `Fx|Reverb`.
    pub sub_categories: String,
    pub kind: PluginKind,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("cannot load {path}: {reason}")]
    Load { path: PathBuf, reason: String },
    #[error("{0} has no VST3 plugin factory")]
    NoFactory(PathBuf),
    #[error("the plugin has no class {0}")]
    NoClass(String),
    #[error("the plugin failed to {0}")]
    Call(&'static str),
    #[error("the plugin does not support {0}")]
    Unsupported(&'static str),
    #[error("invalid plugin state")]
    State,
}
