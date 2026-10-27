//! A hosted plugin, in LimeShot's process or in a bridge process of its own
//! ([`crate::bridge`], Unix only), behind one interface.

use crate::{NoteEvent, ParamInfo, PluginError, Transport, Vst3Editor, Vst3Instance, Vst3Processor, VstPreset};

/// The control side of a hosted plugin.
pub enum PluginInstance {
    Local(Vst3Instance),
    #[cfg(unix)]
    Bridged(crate::bridge::BridgedInstance),
}

/// The audio side of a hosted plugin. Moved once, into a box, so the variants' sizes do not
/// matter.
#[allow(clippy::large_enum_variant)]
pub enum PluginProcessor {
    Local(Vst3Processor),
    #[cfg(unix)]
    Bridged(crate::bridge::BridgedProcessor),
}

macro_rules! each {
    ($self:expr, $p:ident => $e:expr) => {
        match $self {
            Self::Local($p) => $e,
            #[cfg(unix)]
            Self::Bridged($p) => $e,
        }
    };
}

impl PluginInstance {
    /// Whether the plugin runs in a process of its own.
    pub fn is_bridged(&self) -> bool {
        !matches!(self, Self::Local(_))
    }

    /// Whether the plugin's process crashed or hung (never for a plugin in LimeShot's process).
    pub fn crashed(&self) -> bool {
        match self {
            Self::Local(_) => false,
            #[cfg(unix)]
            Self::Bridged(b) => b.crashed(),
        }
    }

    pub fn params(&self) -> &[ParamInfo] {
        each!(self, p => p.params())
    }

    pub fn param(&self, id: u32) -> f64 {
        each!(self, p => p.param(id))
    }

    pub fn set_param(&mut self, id: u32, value: f64) {
        each!(self, p => p.set_param(id, value))
    }

    pub fn show_param(&mut self, id: u32, value: f64) {
        each!(self, p => p.show_param(id, value))
    }

    pub fn param_text(&self, id: u32, value: f64) -> String {
        each!(self, p => p.param_text(id, value))
    }

    /// Edits made in the plugin's own editor (a bridged plugin has none in LimeShot).
    pub fn take_editor_edits(&mut self) -> Vec<(u32, f64)> {
        match self {
            Self::Local(p) => p.take_editor_edits(),
            #[cfg(unix)]
            Self::Bridged(_) => Vec::new(),
        }
    }

    pub fn take_restart(&mut self) -> bool {
        each!(self, p => p.take_restart())
    }

    pub fn editor_gesture_active(&self) -> bool {
        match self {
            Self::Local(p) => p.editor_gesture_active(),
            #[cfg(unix)]
            Self::Bridged(_) => false, // Bridged plugins have no native editor here.
        }
    }

    pub fn latency_samples(&self) -> u32 {
        each!(self, p => p.latency_samples())
    }

    pub fn state(&self) -> Result<Vec<u8>, PluginError> {
        each!(self, p => p.state())
    }

    pub fn set_state(&mut self, blob: &[u8]) -> Result<(), PluginError> {
        each!(self, p => p.set_state(blob))
    }

    pub fn program_names(&self) -> Vec<String> {
        match self {
            Self::Local(p) => p.program_names(),
            #[cfg(unix)]
            Self::Bridged(b) => b.programs().0,
        }
    }

    pub fn program(&self) -> Option<usize> {
        match self {
            Self::Local(p) => p.program(),
            #[cfg(unix)]
            Self::Bridged(b) => b.programs().1,
        }
    }

    pub fn set_program(&mut self, index: usize) {
        each!(self, p => p.set_program(index))
    }

    pub fn load_preset(&mut self, preset: &VstPreset) -> Result<(), PluginError> {
        each!(self, p => p.load_preset(preset))
    }

    pub fn preset(&self) -> Result<VstPreset, PluginError> {
        each!(self, p => p.preset())
    }

    /// The plugin's own editor. A bridged plugin's editor would live in its own process, which
    /// LimeShot cannot show yet.
    pub fn create_editor(&self) -> Option<Vst3Editor> {
        match self {
            Self::Local(p) => p.create_editor(),
            #[cfg(unix)]
            Self::Bridged(_) => None,
        }
    }
}

impl PluginProcessor {
    /// Select a routed mixer key; zero selects the main input.
    pub fn set_sidechain_source(&mut self, source: usize) {
        each!(self,p => p.set_sidechain_source(source));
    }
    /// Selected mixer key source; zero means the processor main input.
    pub fn sidechain_source(&self) -> usize {
        each!(self,p => p.sidechain_source())
    }
    /// Whether the instrument implements the native additive balance protocol.
    pub fn uses_channel_pan(&self) -> bool {
        each!(self,p => p.uses_channel_pan())
    }
    /// Copy completed block statistics without retaining borrowed event pointers.
    pub fn telemetry(&self) -> limeshot_dsp::PluginTelemetry {
        each!(self,p => p.telemetry())
    }
    /// Process stereo audio with timestamped events, transport and optional detector input.
    pub fn process_with_sidechain(&mut self, buf: &mut [f32], notes: &[NoteEvent], t: &Transport, key: Option<&[f32]>) {
        each!(self,p => p.process_with_sidechain(buf,notes,t,key));
    }
    pub fn set_param_at(&mut self, id: u32, offset: u32, value: f64) {
        each!(self, p => p.set_param_at(id, offset, value))
    }

    pub fn restart(&mut self) {
        each!(self, p => p.restart())
    }

    pub fn latency(&self) -> u32 {
        each!(self, p => p.latency())
    }

    pub fn process(&mut self, buf: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        each!(self, p => p.process(buf, notes, transport))
    }
}

impl limeshot_dsp::effects::Effect for PluginProcessor {
    /// Selected mixer key source; zero means the processor main input.
    fn sidechain_source(&self) -> usize {
        self.sidechain_source()
    }
    /// Maximum dynamics reduction from the preceding block, in dB.
    fn gain_reduction_db(&self) -> f32 {
        self.telemetry().reduction_db
    }
    /// Use separate detector audio without mixing the key into the audible output.
    fn process_sidechain(&mut self, buf: &mut [f32], key: &[f32]) {
        each!(self,p => limeshot_dsp::effects::Effect::process_sidechain(p,buf,key));
    }
    fn set_params(&mut self, _params: &[f32]) {}
    fn process(&mut self, buf: &mut [f32]) {
        each!(self, p => limeshot_dsp::effects::Effect::process(p, buf))
    }
    fn reset(&mut self) {
        self.restart();
    }
    fn set_transport(&mut self, transport: &Transport) {
        each!(self, p => limeshot_dsp::effects::Effect::set_transport(p, transport))
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.set_param_at(id, offset, value);
    }
    fn latency(&self) -> u32 {
        PluginProcessor::latency(self)
    }
}

impl limeshot_dsp::Instrument for PluginProcessor {
    /// Whether the instrument implements the native additive balance protocol.
    fn uses_channel_pan(&self) -> bool {
        self.uses_channel_pan()
    }
    /// Actual reported voices, including releases; plugins without telemetry contribute zero.
    fn active_voices(&self) -> usize {
        self.telemetry().voices as usize
    }
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        each!(self, p => limeshot_dsp::Instrument::process(p, out, notes, transport))
    }
    fn reset(&mut self) {
        self.restart();
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.set_param_at(id, offset, value);
    }
    fn latency(&self) -> u32 {
        PluginProcessor::latency(self)
    }
}
