//! Lime Comp VST3. All processing and editor code lives in lime-native.
use lime_native::LimePlugin;
use nice_plug::prelude::*;

nice_export_vst3!(LimePlugin<7>);
