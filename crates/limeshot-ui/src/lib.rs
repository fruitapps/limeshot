//! The LimeShot user interface, built with egui.
//!
//! The UI is thin: panels read the [`limeshot_core::Project`] and change it only through
//! [`limeshot_core::Command`]s, and drive the engine through [`limeshot_audio::Player`] messages.
//! See `docs/architecture/ui.md`.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod app;
pub mod automation_ui;
pub mod browser;
pub mod channel_rack;
pub mod dialogs;
pub mod effects_ui;
pub mod icons;
pub mod inspector;
pub mod menus;
pub mod mixer;
pub mod piano_roll;
pub mod playlist;
pub mod settings;
pub mod shortcuts;
pub mod theme;
pub mod toolbar;
pub mod widgets;
pub mod workspace;

pub use app::LimeShotApp;
pub use settings::Settings;
