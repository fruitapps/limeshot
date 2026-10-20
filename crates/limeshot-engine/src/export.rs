//! Offline rendering (bounce) and WAV export. Uses the same [`Renderer`] as realtime playback.

use crate::renderer::{PlayMode, Renderer};
use limeshot_core::Timeline;
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

/// Render to interleaved stereo f32.
pub fn render_offline(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
) -> Result<(Vec<f32>, RenderReport), ExportError> {
    let length = match settings.range {
        ExportRange::Song => snapshot.song.length,
        ExportRange::Pattern => snapshot.pattern.length,
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
    r.rearrange();
    r.set_mode(match settings.range {
        ExportRange::Song => PlayMode::Song,
        ExportRange::Pattern => PlayMode::Pattern,
    });
    r.set_song_loop(false);
    r.locate(0);
    r.play();
    let music = Timeline::new(tempo, sample_rate).sample_of_tick(length) as usize;
    let tail_max = (settings.max_tail_secs.max(0.0) * sample_rate as f32) as usize;
    let block = settings.block.clamp(1, crate::MAX_BLOCK);
    let mut out = Vec::with_capacity(2 * (music + tail_max.min(sample_rate as usize)));
    let mut buf = vec![0.0f32; 2 * block];
    // The music itself: exactly `music` frames.
    while out.len() / 2 < music {
        let n = block.min(music - out.len() / 2);
        r.render(&mut buf[..2 * n]);
        out.extend_from_slice(&buf[..2 * n]);
    }
    r.pause();
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
    let (audio, report) = render_offline(snapshot, settings)?;
    write_wav(path, report.sample_rate, &audio, depth)?;
    Ok(report)
}
