//! Decoded audio held in memory, WAV reading/writing, and reading MP3, FLAC, Ogg Vorbis and AIFF
//! (symphonia).

use std::path::{Path, PathBuf};

/// Samples longer than this are refused (10 minutes at 192 kHz stereo is ~880 MB of f32).
pub const MAX_FRAMES: usize = 192_000 * 60 * 10;

/// Stereo audio as interleaved f32 (`[l0, r0, l1, r1, …]`). Mono files are duplicated to both
/// sides; files with more than two channels keep the first two.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleData {
    pub sample_rate: u32,
    pub data: Vec<f32>,
    /// Where it was loaded from (for display), if anywhere.
    pub source: Option<PathBuf>,
}

impl SampleData {
    pub fn from_stereo(sample_rate: u32, data: Vec<f32>) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            data,
            source: None,
        }
    }

    pub fn frames(&self) -> usize {
        self.data.len() / 2
    }

    pub fn duration_secs(&self) -> f64 {
        self.frames() as f64 / f64::from(self.sample_rate.max(1))
    }

    /// The stereo frame at `i`, or silence past the ends.
    #[inline]
    pub fn frame(&self, i: usize) -> (f32, f32) {
        match (self.data.get(2 * i), self.data.get(2 * i + 1)) {
            (Some(&l), Some(&r)) => (l, r),
            _ => (0.0, 0.0),
        }
    }

    /// Linear-interpolated frame at fractional position `pos`.
    #[inline]
    pub fn frame_at(&self, pos: f64) -> (f32, f32) {
        if pos < 0.0 {
            return (0.0, 0.0);
        }
        let i = pos as usize;
        let f = (pos - i as f64) as f32;
        let (l0, r0) = self.frame(i);
        let (l1, r1) = self.frame(i + 1);
        (l0 + (l1 - l0) * f, r0 + (r1 - r0) * f)
    }

    /// The largest absolute sample value.
    pub fn peak(&self) -> f32 {
        self.data.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SampleError {
    #[error("{path}: {message}")]
    Decode { path: PathBuf, message: String },
    #[error("{path}: exceeds the {max_frames} frame sample limit")]
    TooLong { path: PathBuf, max_frames: usize },
    #[error("{path}: {source}")]
    Write { path: PathBuf, source: hound::Error },
}

/// Read a WAV file (8/16/24/32-bit integer or 32-bit float PCM).
pub fn load_wav(path: &Path) -> Result<SampleData, SampleError> {
    load_wav_bounded(path, MAX_FRAMES)
}

fn load_wav_bounded(path: &Path, max_frames: usize) -> Result<SampleData, SampleError> {
    let err = |e: hound::Error| SampleError::Decode {
        path: path.to_path_buf(),
        message: e.to_string(),
    };
    let mut reader = hound::WavReader::open(path).map_err(err)?;
    let spec = reader.spec();
    let channels = usize::from(spec.channels.max(1));
    let frames = reader.duration() as usize;
    if frames > max_frames {
        return Err(SampleError::TooLong {
            path: path.to_path_buf(),
            max_frames,
        });
    }
    // Decode directly to stereo: never retain a second, potentially multichannel PCM buffer.
    let mut data = Vec::with_capacity(frames * 2);
    let mut push = |index: usize, v: f32| {
        let clean = if v.is_finite() { v } else { 0.0 };
        if index.is_multiple_of(channels) {
            data.push(clean);
            if channels == 1 {
                data.push(clean);
            }
        } else if index % channels == 1 {
            data.push(clean);
        }
    };
    match spec.sample_format {
        hound::SampleFormat::Float => {
            for (i, value) in reader.samples::<f32>().enumerate() {
                push(i, value.map_err(err)?);
            }
        }
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (u32::from(spec.bits_per_sample.clamp(1, 32)) - 1)) as f32;
            for (i, value) in reader.samples::<i32>().enumerate() {
                push(i, value.map_err(err)? as f32 * scale);
            }
        }
    }
    Ok(SampleData {
        sample_rate: spec.sample_rate.max(1),
        data,
        source: Some(path.to_path_buf()),
    })
}

/// The sample file extensions LimeShot reads (lower case).
pub const AUDIO_EXTENSIONS: [&str; 6] = ["wav", "mp3", "flac", "ogg", "aif", "aiff"];

/// Whether `path` has one of [`AUDIO_EXTENSIONS`].
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| AUDIO_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// Read a sample in any supported format: WAV with hound, MP3, FLAC, Ogg Vorbis and AIFF with
/// symphonia.
pub fn load_audio(path: &Path) -> Result<SampleData, SampleError> {
    load_audio_bounded(path, MAX_FRAMES)
}

/// Decode at most `max_frames` stereo frames. Reject declared oversized files before decoding,
/// and check every compressed packet before appending PCM when duration is unknown.
pub fn load_audio_bounded(path: &Path, max_frames: usize) -> Result<SampleData, SampleError> {
    let max_frames = max_frames.min(MAX_FRAMES);
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("wav")) {
        load_wav_bounded(path, max_frames)
    } else {
        load_compressed(path, max_frames)
    }
}

fn load_compressed(path: &Path, max_frames: usize) -> Result<SampleData, SampleError> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    let fail = |message: String| SampleError::Decode {
        path: path.to_path_buf(),
        message,
    };
    let file = std::fs::File::open(path).map_err(|e| fail(e.to_string()))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| fail(e.to_string()))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| fail("no audio track".into()))?;
    if track.codec_params.n_frames.is_some_and(|n| n > max_frames as u64) {
        return Err(SampleError::TooLong {
            path: path.to_path_buf(),
            max_frames,
        });
    }
    let track_id = track.id;
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| fail(e.to_string()))?;
    let mut data: Vec<f32> = Vec::new();
    let mut buf: Option<SampleBuffer<f32>> = None;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(Error::ResetRequired) => break,
            Err(e) => return Err(fail(e.to_string())),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // A damaged frame is skipped, as players do.
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(fail(e.to_string())),
        };
        if data.len() / 2 + decoded.frames() > max_frames {
            return Err(SampleError::TooLong {
                path: path.to_path_buf(),
                max_frames,
            });
        }
        let spec = *decoded.spec();
        sample_rate = spec.rate;
        let channels = spec.channels.count().max(1);
        let b = buf.get_or_insert_with(|| SampleBuffer::new(decoded.capacity() as u64, spec));
        if b.capacity() < decoded.capacity() * channels {
            *b = SampleBuffer::new(decoded.capacity() as u64, spec);
        }
        b.copy_interleaved_ref(decoded);
        for frame in b.samples().chunks_exact(channels) {
            let l = frame.first().copied().unwrap_or(0.0);
            let r = if channels > 1 {
                frame.get(1).copied().unwrap_or(l)
            } else {
                l
            };
            let clean = |v: f32| if v.is_finite() { v } else { 0.0 };
            data.push(clean(l));
            data.push(clean(r));
        }
    }
    if data.is_empty() {
        return Err(fail("no audio in the file".into()));
    }
    Ok(SampleData {
        sample_rate: sample_rate.max(1),
        data,
        source: Some(path.to_path_buf()),
    })
}

/// Output sample format for [`write_wav`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitDepth {
    Int16,
    #[default]
    Int24,
    Float32,
}

impl BitDepth {
    pub const ALL: [BitDepth; 3] = [BitDepth::Int16, BitDepth::Int24, BitDepth::Float32];

    pub fn label(self) -> &'static str {
        match self {
            BitDepth::Int16 => "16-bit PCM",
            BitDepth::Int24 => "24-bit PCM",
            BitDepth::Float32 => "32-bit float",
        }
    }
}

/// Write interleaved stereo f32 as a WAV file. Integer formats are clipped to full scale
/// (no dithering yet).
pub fn write_wav(path: &Path, sample_rate: u32, interleaved: &[f32], depth: BitDepth) -> Result<(), SampleError> {
    let err = |source| SampleError::Write {
        path: path.to_path_buf(),
        source,
    };
    let (bits, format) = match depth {
        BitDepth::Int16 => (16, hound::SampleFormat::Int),
        BitDepth::Int24 => (24, hound::SampleFormat::Int),
        BitDepth::Float32 => (32, hound::SampleFormat::Float),
    };
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: bits,
        sample_format: format,
    };
    let mut w = hound::WavWriter::create(path, spec).map_err(err)?;
    match depth {
        BitDepth::Float32 => {
            for &s in interleaved {
                w.write_sample(s).map_err(err)?;
            }
        }
        BitDepth::Int16 => {
            for &s in interleaved {
                w.write_sample((s.clamp(-1.0, 1.0) * 32767.0).round() as i16)
                    .map_err(err)?;
            }
        }
        BitDepth::Int24 => {
            for &s in interleaved {
                w.write_sample((s.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32)
                    .map_err(err)?;
            }
        }
    }
    w.finalize().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_round_trip_all_depths() {
        let dir = tempfile::tempdir().unwrap();
        let data: Vec<f32> = (0..2000).map(|i| ((i as f32) * 0.01).sin() * 0.5).collect();
        for depth in BitDepth::ALL {
            let p = dir.path().join(format!("{depth:?}.wav"));
            write_wav(&p, 44_100, &data, depth).unwrap();
            let s = load_wav(&p).unwrap();
            assert_eq!(s.sample_rate, 44_100);
            assert_eq!(s.frames(), 1000);
            let tol = if depth == BitDepth::Int16 { 1e-4 } else { 1e-6 };
            for (a, b) in s.data.iter().zip(&data) {
                assert!((a - b).abs() < tol, "{depth:?}");
            }
        }
    }

    #[test]
    fn mono_is_duplicated() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("m.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 8000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        for v in [0i16, 16384, -16384] {
            w.write_sample(v).unwrap();
        }
        w.finalize().unwrap();
        let s = load_wav(&p).unwrap();
        assert_eq!(s.data, vec![0.0, 0.0, 0.5, 0.5, -0.5, -0.5]);
    }

    #[test]
    fn garbage_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("bad.wav");
        std::fs::write(&p, b"RIFF\x10\x00\x00\x00WAVEjunkjunk").unwrap();
        assert!(load_wav(&p).is_err());
        assert!(load_wav(&dir.path().join("missing.wav")).is_err());
    }

    #[test]
    fn interpolation() {
        let s = SampleData::from_stereo(10, vec![0.0, 0.0, 1.0, -1.0]);
        assert_eq!(s.frame_at(0.5), (0.5, -0.5));
        assert_eq!(s.frame_at(5.0), (0.0, 0.0));
        assert_eq!(s.frame_at(-1.0), (0.0, 0.0));
    }
}

#[cfg(test)]
mod bounded_tests {
    use super::*;
    #[test]
    fn bounded_decoders_reject_declared_wav_and_compressed_duration() {
        let dir = tempfile::tempdir().unwrap();
        let wav = dir.path().join("bounded.wav");
        write_wav(&wav, 48000, &[0.2; 256], BitDepth::Float32).unwrap();
        assert!(matches!(
            load_audio_bounded(&wav, 127),
            Err(SampleError::TooLong { max_frames: 127, .. })
        ));
        assert_eq!(load_audio_bounded(&wav, 128).unwrap().frames(), 128);
        let mp3 = dir.path().join("bounded.mp3");
        let mut frame = vec![0; 417];
        frame[..4].copy_from_slice(&[0xff, 0xfb, 0x90, 0]);
        std::fs::write(&mp3, frame.repeat(8)).unwrap();
        assert!(matches!(
            load_audio_bounded(&mp3, 1000),
            Err(SampleError::TooLong { .. })
        ));
        assert!(load_audio_bounded(&mp3, 10000).unwrap().frames() > 1000);
    }
}
