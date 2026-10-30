//! Offline rendering (bounce) and audio export. Uses the same [`Renderer`] as realtime playback.

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
    #[error("MP3 needs a sample rate of 32000, 44100 or 48000 Hz")]
    Mp3SampleRate,
    #[error("unsupported MP3 bitrate: {0} kbps")]
    Mp3Bitrate(u32),
    #[error("MP3 encoding failed: {0}")]
    Mp3(String),
    #[error("could not write audio: {0}")]
    Io(#[from] std::io::Error),
}

/// MPEG-1 constant bitrates; reject unsupported values instead of silently rounding them.
pub const MP3_BITRATES: &[u32] = &[32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
pub const MP3_SAMPLE_RATES: &[u32] = &[32_000, 44_100, 48_000];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Wav(BitDepth),
    Mp3 { bitrate_kbps: u32 },
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav(_) => "wav",
            Self::Mp3 { .. } => "mp3",
        }
    }

    pub fn validate(self, sample_rate: u32) -> Result<(), ExportError> {
        if let Self::Mp3 { bitrate_kbps } = self {
            if !MP3_SAMPLE_RATES.contains(&sample_rate) {
                return Err(ExportError::Mp3SampleRate);
            }
            if !MP3_BITRATES.contains(&bitrate_kbps) {
                return Err(ExportError::Mp3Bitrate(bitrate_kbps));
            }
        }
        Ok(())
    }
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
        ExportRange::Selection { from, to } => (from, to.min(snapshot.song.length).saturating_sub(from)),
    };
    if length == 0 {
        return Err(ExportError::EmptySong);
    }
    let tempo = snapshot.tempo;
    let mut r = Renderer::new(settings.sample_rate, snapshot.clone());
    let sample_rate = r.sample_rate();
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
    r.set_pattern_loop_range(None);
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
        if left == 0 || r.finished() {
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
    export_audio_with(snapshot, settings, ExportFormat::Wav(depth), path, processors)
}

/// Render the same mix for either format. Encoding and file I/O run off the audio thread.
pub fn export_audio_with(
    snapshot: Arc<Snapshot>,
    settings: &RenderSettings,
    format: ExportFormat,
    path: &Path,
    processors: Processors,
) -> Result<RenderReport, ExportError> {
    format.validate(settings.sample_rate)?;
    let (mut audio, report) = render_offline_with(snapshot, settings, processors)?;
    match format {
        ExportFormat::Wav(depth) => write_wav(path, report.sample_rate, &audio, depth)?,
        ExportFormat::Mp3 { bitrate_kbps } => write_mp3(path, report.sample_rate, bitrate_kbps, &mut audio)?,
    }
    Ok(report)
}

/// Encode stereo `audio` as constant-bitrate MP3 and atomically replace `path`.
fn write_mp3(path: &Path, sample_rate: u32, bitrate_kbps: u32, audio: &mut [f32]) -> Result<(), ExportError> {
    use std::io::{BufWriter, Write};
    // Do not feed non-finite plugin output or overs into the encoder's quantizer.
    for sample in audio.iter_mut() {
        *sample = if sample.is_finite() {
            sample.clamp(-1.0, 1.0)
        } else {
            0.0
        };
    }
    let mut encoder = rusty_mp3::Mp3Encoder::new(rusty_mp3::Mp3EncoderConfig {
        bitrate_kbps,
        vbr_quality: None,
    });
    encoder
        .push_pcm_f32(audio, 2, sample_rate)
        .map_err(|error| ExportError::Mp3(error.to_string()))?;
    // Drain after finish so the first packet has the final delay/padding (gapless) tag.
    encoder.finish();
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let existing_permissions = std::fs::metadata(path).ok().map(|meta| meta.permissions());
    #[cfg(unix)]
    let default_permissions = {
        let probe = tempfile::Builder::new()
            .permissions(std::os::unix::fs::PermissionsExt::from_mode(0o666))
            .tempfile_in(parent)?;
        probe.as_file().metadata()?.permissions()
    };
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        loop {
            match encoder.next_packet() {
                Ok(packet) => writer.write_all(&packet)?,
                Err(rusty_mp3::Error::Eof) => break,
                Err(error) => return Err(ExportError::Mp3(error.to_string())),
            }
        }
        writer.flush()?;
    }
    temporary.as_file().sync_all()?;
    if let Some(permissions) = existing_permissions {
        let _ = temporary.as_file().set_permissions(permissions);
    } else {
        #[cfg(unix)]
        let _ = temporary.as_file().set_permissions(default_permissions);
    }
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mp3_encodes_stereo_at_each_rate_and_bitrate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.mp3");
        for &rate in MP3_SAMPLE_RATES {
            for &bitrate in MP3_BITRATES {
                let frames = rate as usize / 4 + 17; // A partial final MPEG frame.
                let mut audio: Vec<f32> = (0..frames)
                    .flat_map(|i| {
                        let t = i as f32 / rate as f32;
                        [
                            0.5 * (t * 440.0 * std::f32::consts::TAU).sin(),
                            0.3 * (t * 880.0 * std::f32::consts::TAU).sin(),
                        ]
                    })
                    .collect();
                write_mp3(&path, rate, bitrate, &mut audio).unwrap();
                let bytes = std::fs::read(&path).unwrap();
                assert!(bytes.windows(4).any(|w| w == b"Info"), "final seek/delay tag");
                // Symphonia is independent of the encoder. It currently retains codec padding.
                let decoded = limeshot_dsp::load_audio(&path).unwrap();
                assert_eq!(decoded.sample_rate, rate);
                assert!(
                    decoded.frames() >= frames && decoded.frames() < frames + 3456,
                    "rate {rate}, bitrate {bitrate}: {} vs {frames}",
                    decoded.frames()
                );
                assert!(decoded.data.iter().all(|s| s.is_finite()));
                let channel_energy = |channel: usize, hz: f32| {
                    let mut re = 0.0f64;
                    let mut im = 0.0f64;
                    for (i, frame) in decoded.data.as_chunks::<2>().0.iter().enumerate() {
                        let angle = i as f64 * f64::from(hz) * std::f64::consts::TAU / f64::from(rate);
                        re += f64::from(frame[channel]) * angle.cos();
                        im += f64::from(frame[channel]) * angle.sin();
                    }
                    (re * re + im * im).sqrt() / decoded.frames() as f64
                };
                assert!(channel_energy(0, 440.0) > 0.1, "left tone {rate}/{bitrate}");
                assert!(channel_energy(1, 880.0) > 0.05, "right tone {rate}/{bitrate}");
                assert!(channel_energy(0, 880.0) < channel_energy(0, 440.0) * 0.2, "stereo left");
                assert!(
                    channel_energy(1, 440.0) < channel_energy(1, 880.0) * 0.2,
                    "stereo right"
                );
                // Verify the first audio frame's MPEG-1 bitrate/rate header, not just decoder success.
                let header = rusty_mp3::encoder_header(rate, 2, bitrate).unwrap();
                let tag_header = rusty_mp3::header::FrameHeader::parse(bytes[..4].try_into().unwrap()).unwrap();
                let offset = tag_header.frame_size();
                let bitrate_index = (bytes[offset + 2] >> 4) as usize;
                let rate_index = ((bytes[offset + 2] >> 2) & 3) as usize;
                assert_eq!(
                    [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320][bitrate_index],
                    bitrate
                );
                assert_eq!([44100, 48000, 32000][rate_index], rate);
                let expected = frames as f64 / f64::from(rate) * f64::from(bitrate) * 125.0;
                assert!(
                    (bytes.len() as f64 - expected).abs() < 4.0 * header.frame_size() as f64,
                    "CBR size {rate}/{bitrate}: {} bytes",
                    bytes.len()
                );
            }
        }
    }

    #[test]
    fn mp3_short_silent_and_nonfinite_inputs_and_write_failures() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("short.mp3");
        for frames in [1, 23, 1152, 1153] {
            let mut audio = vec![0.0; frames * 2];
            audio[0] = f32::NAN;
            audio[1] = f32::INFINITY;
            write_mp3(&path, 44100, 320, &mut audio).unwrap();
            let decoded = limeshot_dsp::load_audio(&path).unwrap();
            assert!(decoded.data.iter().all(|v| v.is_finite() && v.abs() < 0.001));
        }
        assert!(write_mp3(&dir.path().join("absent/out.mp3"), 44100, 128, &mut [0.0; 2]).is_err());
        // A rename failure leaves no temporary file behind.
        let target = dir.path().join("directory.mp3");
        std::fs::create_dir(&target).unwrap();
        assert!(write_mp3(&target, 44100, 128, &mut [0.0; 2]).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
            write_mp3(&path, 44100, 128, &mut [0.0; 2]).unwrap();
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o640);
        }
    }

    #[test]
    fn invalid_mp3_settings_preserve_existing_output() {
        let project = limeshot_core::Project::new();
        let snap = Arc::new(limeshot_sequencer::compile(
            &project,
            project.patterns[0].id,
            &limeshot_sequencer::NoSamples,
        ));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.mp3");
        std::fs::write(&path, b"keep this").unwrap();
        for (rate, bitrate) in [(96000, 320), (44100, 123), (0, 320)] {
            let result = export_audio_with(
                snap.clone(),
                &RenderSettings {
                    sample_rate: rate,
                    ..Default::default()
                },
                ExportFormat::Mp3 { bitrate_kbps: bitrate },
                &path,
                Processors::default(),
            );
            assert!(matches!(
                result,
                Err(ExportError::Mp3SampleRate | ExportError::Mp3Bitrate(_))
            ));
            assert_eq!(std::fs::read(&path).unwrap(), b"keep this");
        }
    }
}
