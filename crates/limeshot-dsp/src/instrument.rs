//! Instruments that render a whole channel from note events (hosted plugins), as opposed to the
//! built-in per-note voices.

/// A note at a frame offset inside the block being rendered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteEvent {
    pub offset: u32,
    pub kind: NoteEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteEventKind {
    /// `velocity` is 0–1; `id` pairs the note with its `Off`.
    On {
        key: u8,
        velocity: f32,
        id: i32,
    },
    Off {
        key: u8,
        id: i32,
    },
}

/// The host's transport at the first frame of a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transport {
    pub playing: bool,
    pub tempo: f64,
    pub sample_pos: i64,
    /// Position in quarter notes.
    pub ppq_pos: f64,
    pub time_sig: (i32, i32),
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            playing: false,
            tempo: 120.0,
            sample_pos: 0,
            ppq_pos: 0.0,
            time_sig: (4, 4),
        }
    }
}

/// A sound source that renders a channel's notes itself. Implementations must not allocate,
/// lock or block in `process`.
pub trait Instrument: Send {
    /// Render `out.len() / 2` frames of interleaved stereo, overwriting `out`. `notes` are
    /// sorted by offset.
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], transport: &Transport);
    /// Silence every note and clear tails.
    fn reset(&mut self);
    /// Set a hosted plugin's parameter `id` (normalised) at frame `offset` of the next `process`
    /// call (automation). Only plugins have such parameters.
    fn set_plugin_param(&mut self, _id: u32, _offset: u32, _value: f64) {}
}
