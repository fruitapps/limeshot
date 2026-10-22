//! Offline rendering (bounce) and WAV export. Uses the same [`Renderer`] as realtime playback.

use crate::renderer::{PlayMode, Renderer};
use limeshot_core::{Tick, Timeline};
use limeshot_dsp::{BitDepth, SampleError, write_wav};
use limeshot_sequencer::Snapshot;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportRange {
    /// The whole Playlist arrangement.
    #[default]
    Song,
    /// One pass of the current pattern.
    Pattern,
    /// The Playlist's time selection, ticks `from..to` of the song (FL Studio renders the
    /// selection when there is one).
    Selection { from: Tick, to: Tick },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSettings {
    pub sample_rate: u32,
    pub range: ExportRange,
    /// Longest time to let notes and effect tails ring out after the end, in seconds.
    pub max_tail_secs: f32,
    /// Block size used for rendering (results do not depend on it; tests vary it).
    pub block: usize,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            range: ExportRange::Song,
            max_tail_secs: 4.0,
            block: 512,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("the Playlist is empty: place a pattern on it, or export the pattern instead")]
    EmptySong,
    #[error(transparent)]
    Write(#[from] SampleError),
}

/// What an export produced.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderReport {
    pub frames: usize,
    pub sample_rate: u32,
    /// Frames up to the end of the music (without the tail).
    pub music_frames: usize,
    pub peak: f32,
}

impl RenderReport {
    pub fn seconds(&self) -> f64 {
        self.frames as f64 / f64::from(self.sample_rate.max(1))
    }
}

/// Processors the host built for plugin effects (by effect id) and plugin channels (by channel
/// id), for an offline render. They must be instances of their own, not the realtime ones.
#[derive(Default)]
pub struct Processors {
    pub effects: Vec<(u32, Box<dyn limeshot_dsp::effects::Effect>)>,
    pub instruments: Vec<(u32, Box<dyn limeshot_dsp::Instrument>)>,
}

/// Render to interleaved stereo f32 (plugin slots and channels stay silent; see
/// [`render_offline_with`]).
pub fn render_offline(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
) -> Result<(Vec<f32>, RenderReport), ExportError> {
    render_offline_with(snapshot, settings, Processors::default())
}

/// Render to interleaved stereo f32 with the host's plugin processors.
pub fn render_offline_with(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
    processors: Processors,
) -> Result<(Vec<f32>, RenderReport), ExportError> {
    let (start, length) = match settings.range {
        ExportRange::Song => (0, snapshot.song.length),
        ExportRange::Pattern => (0, snapshot.pattern.length),
        ExportRange::Selection { from, to } => (from, to.saturating_sub(from)),
    };
    if length == 0 {
        return Err(ExportError::EmptySong);
    }
    let tempo = snapshot.tempo;
    let mut r = Renderer::new(settings.sample_rate, snapshot.clone());
    let sample_rate = r.sample_rate();
    for (id, fx) in crate::fx::build_all(&snapshot, sample_rate) {
        r.adopt_effect(id, fx);
    }
    for (id, fx) in processors.effects {
        r.adopt_effect(id, fx);
    }
    for (channel, inst) in processors.instruments {
        r.adopt_instrument(channel, inst);
    }
    r.rearrange();
    r.set_mode(match settings.range {
        ExportRange::Song | ExportRange::Selection { .. } => PlayMode::Song,
        ExportRange::Pattern => PlayMode::Pattern,
    });
    r.set_song_loop(false);
    r.set_loop_range(None);
    r.locate(start);
    r.play();
    let tl = Timeline::new(tempo, sample_rate);
    // Without tempo automation this is the music's exact length; with it, the estimate only
    // sizes the buffer and the loop below runs to the end at whatever tempo plays.
    let estimate = (tl.sample_of_tick(start + length) - tl.sample_of_tick(start)) as usize;
    let end = start + length;
    let tail_max = (settings.max_tail_secs.max(0.0) * sample_rate as f32) as usize;
    let block = settings.block.clamp(1, crate::MAX_BLOCK);
    let mut out = Vec::with_capacity(2 * (estimate + tail_max.min(sample_rate as usize)));
    let mut buf = vec![0.0f32; 2 * block];
    // Plugin delay compensation delays the whole mix; its first frames are silence, left out as
    // FL Studio's "Trim PDC silence" does, and the music ends that much later.
    let latency = r.latency() as usize;
    let mut skip = latency;
    let mut rendered = 0;
    // The music itself: transport until the end tick, at the tempo playing. The frames left
    // are re-read every block (tempo automation changes them); once a block reaches the end the
    // music is done, before the transport wraps round (Pattern mode loops).
    loop {
        let left = r.frames_until(end) as usize;
        if left == 0 {
            break;
        }
        let n = block.min(left);
        r.render(&mut buf[..2 * n]);
        let s = skip.min(n);
        skip -= s;
        out.extend_from_slice(&buf[2 * s..2 * n]);
        rendered += n;
        if n == left {
            break;
        }
    }
    let music = rendered;
    r.pause();
    // The compensation delay's last frames finish the music.
    let mut flush = latency;
    while flush > 0 {
        let n = block.min(flush);
        r.render(&mut buf[..2 * n]);
        let s = skip.min(n);
        skip -= s;
        out.extend_from_slice(&buf[2 * s..2 * n]);
        flush -= n;
    }
    // The tail: until every voice has finished or the limit is reached.
    let mut tail = 0;
    while tail < tail_max && (r.active_voices() > 0 || r.is_ringing()) {
        let n = block.min(tail_max - tail);
        r.render(&mut buf[..2 * n]);
        out.extend_from_slice(&buf[..2 * n]);
        tail += n;
    }
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    let report = RenderReport {
        frames: out.len() / 2,
        sample_rate,
        music_frames: music,
        peak,
    };
    Ok((out, report))
}

/// Render and write a stereo WAV file.
pub fn export_wav(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
    depth: BitDepth,
    path: &Path,
) -> Result<RenderReport, ExportError> {
    export_wav_with(snapshot, settings, depth, path, Processors::default())
}

/// Render with the host's plugin processors and write a stereo WAV file.
pub fn export_wav_with(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
    depth: BitDepth,
    path: &Path,
    processors: Processors,
) -> Result<RenderReport, ExportError> {
    let (audio, report) = render_offline_with(snapshot, settings, processors)?;
    write_wav(path, report.sample_rate, &audio, depth)?;
    Ok(report)
}
