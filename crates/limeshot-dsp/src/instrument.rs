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
    /// Per-note VST3 pan expression (independent for overlapping voices).
    Pan {
        id: i32,
        pan: f32,
    },
    ChannelPan {
        pan: f32,
    },
    /// Lime Sampler clip start, carried as a versioned SysEx data event.
    Clip {
        key: u8,
        velocity: f32,
        id: i32,
        playback: ClipPlayback,
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
    /// Actual active voices when the instrument reports them; unknown plugins return zero.
    fn uses_channel_pan(&self) -> bool {
        false
    }
    /// Actual reported voices, including releases; plugins without telemetry contribute zero.
    fn active_voices(&self) -> usize {
        0
    }
    /// How many frames the output lags the notes (a plugin's reported latency).
    fn latency(&self) -> u32 {
        0
    }
}

/// Clip timing in output seconds. Source trim and elapsed playback are separate so fades
/// remain relative to the visible clip, including when playback starts inside it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipPlayback {
    pub offset_secs: f64,
    pub elapsed_secs: f64,
    pub remaining_secs: f64,
    pub fade_in_secs: f32,
    pub fade_out_secs: f32,
    pub fade_in_curve: f32,
    pub fade_out_curve: f32,
}

/// Fixed-size, versioned, 7-bit-clean MIDI SysEx. Shared by host and native sampler;
/// no allocation, pointers, platform-sized fields or persisted plugin state involved.
pub const CLIP_MESSAGE_LEN: usize = 96;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipMessage {
    pub key: u8,
    pub velocity: f32,
    pub id: i32,
    pub playback: ClipPlayback,
}
impl ClipMessage {
    /// Encode clip timing, fades and offsets as fixed-size seven-bit-clean noncommercial SysEx.
    pub fn encode(self) -> [u8; CLIP_MESSAGE_LEN] {
        let mut raw = [0u8; 45];
        raw[0] = self.key;
        raw[1..5].copy_from_slice(&self.velocity.to_le_bytes());
        raw[5..9].copy_from_slice(&self.id.to_le_bytes());
        raw[9..17].copy_from_slice(&self.playback.offset_secs.to_le_bytes());
        raw[17..25].copy_from_slice(&self.playback.elapsed_secs.to_le_bytes());
        raw[25..33].copy_from_slice(&self.playback.remaining_secs.to_le_bytes());
        raw[33..37].copy_from_slice(&self.playback.fade_in_secs.to_le_bytes());
        raw[37..41].copy_from_slice(&self.playback.fade_out_secs.to_le_bytes());
        raw[41..43]
            .copy_from_slice(&((self.playback.fade_in_curve.clamp(-1.0, 2.0) * 1000.0).round() as i16).to_le_bytes());
        raw[43..45]
            .copy_from_slice(&((self.playback.fade_out_curve.clamp(-1.0, 2.0) * 1000.0).round() as i16).to_le_bytes());
        let mut out = [0u8; CLIP_MESSAGE_LEN];
        let version = if self.playback.fade_in_curve == 2.0 || self.playback.fade_out_curve == 2.0 {
            3
        } else if self.playback.fade_in_curve == 0.0 && self.playback.fade_out_curve == 0.0 {
            1
        } else {
            2
        };
        out[..5].copy_from_slice(&[0xf0, 0x7d, b'L', b'C', version]);
        for (pair, byte) in out[5..95].as_chunks_mut::<2>().0.iter_mut().zip(raw) {
            pair[0] = byte >> 4;
            pair[1] = byte & 15;
        }
        out[95] = 0xf7;
        out
    }
    /// Validate framing, version, finite values and bounds before accepting a protocol message.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != CLIP_MESSAGE_LEN
            || bytes[..4] != [0xf0, 0x7d, b'L', b'C']
            || !(1..=3).contains(&bytes[4])
            || bytes[95] != 0xf7
            || bytes[5..95].iter().any(|b| *b > 15)
        {
            return None;
        }
        let mut raw = [0u8; 45];
        for (byte, pair) in raw.iter_mut().zip(bytes[5..95].as_chunks::<2>().0.iter()) {
            *byte = pair[0] << 4 | pair[1];
        }
        let f32_at = |i| Some(f32::from_le_bytes(raw.get(i..i + 4)?.try_into().ok()?));
        let f64_at = |i| Some(f64::from_le_bytes(raw.get(i..i + 8)?.try_into().ok()?));
        let playback = ClipPlayback {
            offset_secs: f64_at(9)?,
            elapsed_secs: f64_at(17)?,
            remaining_secs: f64_at(25)?,
            fade_in_secs: f32_at(33)?,
            fade_out_secs: f32_at(37)?,
            fade_in_curve: if bytes[4] >= 2 {
                f32::from(i16::from_le_bytes([raw[41], raw[42]])) / 1000.0
            } else {
                0.0
            },
            fade_out_curve: if bytes[4] >= 2 {
                f32::from(i16::from_le_bytes([raw[43], raw[44]])) / 1000.0
            } else {
                0.0
            },
        };
        if [playback.fade_in_curve, playback.fade_out_curve]
            .iter()
            .any(|v| v.abs() > 1.0 && !(bytes[4] == 3 && *v == 2.0))
        {
            return None;
        }
        let velocity = f32_at(1)?;
        if raw[0] > 127
            || !velocity.is_finite()
            || !(0.0..=1.0).contains(&velocity)
            || [
                playback.offset_secs,
                playback.elapsed_secs,
                playback.remaining_secs,
                f64::from(playback.fade_in_secs),
                f64::from(playback.fade_out_secs),
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0 || *v > 86400.0 * 365.0)
        {
            return None;
        }
        Some(Self {
            key: raw[0],
            velocity,
            id: i32::from_le_bytes(raw[5..9].try_into().ok()?),
            playback,
        })
    }
}

/// Native processor telemetry sent once per block as a noncommercial SysEx message.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PluginTelemetry {
    pub voices: u32,
    pub reduction_db: f32,
}
impl PluginTelemetry {
    /// Encode end-of-block voice count and maximum dynamics reduction into bounded SysEx.
    pub fn encode(self) -> [u8; 22] {
        let mut out = [0; 22];
        out[..5].copy_from_slice(&[0xf0, 0x7d, b'L', b'T', 1]);
        let mut raw = [0; 8];
        raw[..4].copy_from_slice(&self.voices.to_le_bytes());
        raw[4..].copy_from_slice(&self.reduction_db.to_le_bytes());
        for (p, b) in out[5..21].as_chunks_mut::<2>().0.iter_mut().zip(raw) {
            p[0] = b >> 4;
            p[1] = b & 15;
        }
        out[21] = 0xf7;
        out
    }
    /// Validate framing, version, finite values and bounds before accepting a protocol message.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 22
            || bytes[..5] != [0xf0, 0x7d, b'L', b'T', 1]
            || bytes[21] != 0xf7
            || bytes[5..21].iter().any(|b| *b > 15)
        {
            return None;
        }
        let mut raw = [0; 8];
        for (b, p) in raw.iter_mut().zip(bytes[5..21].as_chunks::<2>().0.iter()) {
            *b = p[0] << 4 | p[1];
        }
        let value = Self {
            voices: u32::from_le_bytes(raw[..4].try_into().ok()?),
            reduction_db: f32::from_le_bytes(raw[4..].try_into().ok()?),
        };
        (value.voices <= 128 && value.reduction_db.is_finite() && (0.0..=200.0).contains(&value.reduction_db))
            .then_some(value)
    }
}

/// Encode additive channel balance using a fixed stack buffer.
pub fn channel_pan_message(pan: f32) -> [u8; 14] {
    let mut out = [0; 14];
    out[..5].copy_from_slice(&[0xf0, 0x7d, b'L', b'P', 1]);
    for (pair, b) in out[5..13]
        .as_chunks_mut::<2>()
        .0
        .iter_mut()
        .zip(pan.clamp(-1.0, 1.0).to_le_bytes())
    {
        pair[0] = b >> 4;
        pair[1] = b & 15;
    }
    out[13] = 0xf7;
    out
}
/// Validate native channel balance without retaining the input buffer.
pub fn decode_channel_pan(bytes: &[u8]) -> Option<f32> {
    if bytes.len() != 14
        || bytes[..5] != [0xf0, 0x7d, b'L', b'P', 1]
        || bytes[13] != 0xf7
        || bytes[5..13].iter().any(|b| *b > 15)
    {
        return None;
    }
    let mut raw = [0; 4];
    for (b, p) in raw.iter_mut().zip(bytes[5..13].as_chunks::<2>().0.iter()) {
        *b = p[0] << 4 | p[1];
    }
    let pan = f32::from_le_bytes(raw);
    (pan.is_finite() && (-1.0..=1.0).contains(&pan)).then_some(pan)
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    #[test]
    fn native_event_messages_are_bounded_and_reject_malformed_data() {
        let message = ClipMessage {
            key: 60,
            velocity: 0.8,
            id: 42,
            playback: ClipPlayback {
                offset_secs: 0.25,
                elapsed_secs: 0.1,
                remaining_secs: 0.5,
                fade_in_secs: 0.2,
                fade_out_secs: 0.1,
                fade_in_curve: 0.375,
                fade_out_curve: -0.625,
            },
        };
        let bytes = message.encode();
        assert_eq!(ClipMessage::decode(&bytes), Some(message));
        let mut smooth = message;
        smooth.playback.fade_in_curve = 2.;
        smooth.playback.fade_out_curve = 2.;
        let mut encoded = smooth.encode();
        assert_eq!(encoded[4], 3);
        assert_eq!(ClipMessage::decode(&encoded), Some(smooth));
        encoded[4] = 2;
        assert!(ClipMessage::decode(&encoded).is_none());
        for length in 0..bytes.len() {
            assert!(ClipMessage::decode(&bytes[..length]).is_none());
        }
        let mut bad = bytes;
        bad[5] = 0x80;
        assert!(ClipMessage::decode(&bad).is_none());
        let mut bad = message;
        bad.playback.offset_secs = f64::NAN;
        assert!(ClipMessage::decode(&bad.encode()).is_none());
        for pan in [-1.0, 0.0, 1.0] {
            assert_eq!(decode_channel_pan(&channel_pan_message(pan)), Some(pan));
        }
        let telemetry = PluginTelemetry {
            voices: 128,
            reduction_db: 12.5,
        };
        assert_eq!(PluginTelemetry::decode(&telemetry.encode()), Some(telemetry));
        assert!(
            PluginTelemetry::decode(
                &PluginTelemetry {
                    voices: 129,
                    ..telemetry
                }
                .encode()
            )
            .is_none()
        );
    }
}
