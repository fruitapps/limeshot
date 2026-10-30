//! Decoded audio held in memory, WAV reading/writing, and reading MP3, FLAC, Ogg Vorbis and AIFF
//! (symphonia).

use std::path::{Path, PathBuf};

/// Samples longer than this are refused (10 minutes at 192 kHz stereo is ~880 MB of f32).
pub const MAX_FRAMES: usize = crate::disk::MAX_DISK_FRAMES;

/// Stereo audio as interleaved f32 (`[l0, r0, l1, r1, …]`). Mono files are duplicated to both
/// sides; files with more than two channels keep the first two.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleData {
    pub sample_rate: u32,
    pub data: Vec<f32>,
    pub disk: Option<std::sync::Arc<crate::disk::DiskSample>>,
    /// Where it was loaded from (for display), if anywhere.
    pub source: Option<PathBuf>,
}

impl SampleData {
    pub fn from_stereo(sample_rate: u32, data: Vec<f32>) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            data,
            disk: None,
            source: None,
        }
    }

    pub fn frames(&self) -> usize {
        self.disk.as_ref().map_or(self.data.len() / 2, |d| d.frames)
    }

    pub fn duration_secs(&self) -> f64 {
        self.frames() as f64 / f64::from(self.sample_rate.max(1))
    }

    /// The stereo frame at `i`, or silence past the ends.
    #[inline]
    pub fn frame(&self, i: usize) -> (f32, f32) {
        if let Some(disk) = &self.disk {
            return disk.frame(i);
        }
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

    /// Prepared disk extrema; drawing never triggers page reads for a long sample.
    pub fn overview_range(&self, a: usize, b: usize) -> Option<(f32, f32)> {
        let disk = self.disk.as_ref()?;
        let start = a / crate::disk::PEAK_BLOCK;
        let end = (b / crate::disk::PEAK_BLOCK + 1).min(disk.peaks.len());
        disk.peaks
            .get(start..end)?
            .iter()
            .copied()
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
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
        disk: None,
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
    let mut data = Vec::new();
    let (sample_rate, _) = decode_to(path, max_frames, |_, left, right| {
        data.push(left);
        data.push(right);
        Ok(())
    })?;
    Ok(SampleData {
        sample_rate,
        data,
        source: Some(path.to_path_buf()),
        disk: None,
    })
}

pub(crate) fn decode_to(
    path: &Path,
    max_frames: usize,
    mut emit: impl FnMut(u32, f32, f32) -> Result<(), SampleError>,
) -> Result<(u32, usize), SampleError> {
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
    // Symphonia can panic while probing hostile headers (for example an AIFF rate that
    // rounds to zero in TimeBase). Contain third-party probing failures on the loader thread.
    let probed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        symphonia::default::get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
    }))
    .map_err(|_| fail("audio format probe panicked on an invalid header".into()))?
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
    let mut frames = 0usize;
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
        if frames + decoded.frames() > max_frames {
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
            emit(sample_rate, clean(l), clean(r))?;
            frames += 1;
        }
    }
    if frames == 0 {
        return Err(fail("no audio in the file".into()));
    }
    Ok((sample_rate.max(1), frames))
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

/// Write interleaved stereo f32 as a WAV file. Integer formats are clipped to full scale,
/// without dither (see [`write_wav_dithered`]).
pub fn write_wav(path: &Path, sample_rate: u32, interleaved: &[f32], depth: BitDepth) -> Result<(), SampleError> {
    write_wav_dithered(path, sample_rate, interleaved, depth, None)
}

/// Write interleaved stereo f32 as a WAV file; integer formats get TPDF dither when `dither`
/// holds a seed (see [`WavStream`]).
pub fn write_wav_dithered(
    path: &Path,
    sample_rate: u32,
    interleaved: &[f32],
    depth: BitDepth,
    dither: Option<u64>,
) -> Result<(), SampleError> {
    let err = |source| SampleError::Write {
        path: path.to_path_buf(),
        source,
    };
    let file = std::fs::File::create(path).map_err(|e| err(hound::Error::IoError(e)))?;
    let mut w = WavStream::new(std::io::BufWriter::new(file), sample_rate, depth, dither).map_err(err)?;
    w.write(interleaved).map_err(err)?;
    w.finalize().map_err(err)
}

/// A stereo WAV written block by block (long renders, one file per mixer track).
///
/// Integer formats clip to full scale. With a dither seed they add TPDF dither first:
/// triangular noise of ±1 LSB (the sum of two uniform ±½ LSB values), which turns the
/// quantization error into signal-independent noise so quiet passages and fades do not distort.
/// The noise comes from a seeded generator, so the same render writes the same file. Float
/// output is never dithered.
pub struct WavStream<W: std::io::Write + std::io::Seek> {
    writer: hound::WavWriter<W>,
    depth: BitDepth,
    rng: Option<u64>,
}

impl<W: std::io::Write + std::io::Seek> WavStream<W> {
    pub fn new(out: W, sample_rate: u32, depth: BitDepth, dither: Option<u64>) -> Result<Self, hound::Error> {
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
        Ok(Self {
            writer: hound::WavWriter::new(out, spec)?,
            depth,
            // A zero state would make xorshift return zeros for ever.
            rng: dither.map(|seed| seed | 1),
        })
    }

    /// Append interleaved stereo frames.
    pub fn write(&mut self, interleaved: &[f32]) -> Result<(), hound::Error> {
        let full = match self.depth {
            BitDepth::Float32 => {
                for &s in interleaved {
                    self.writer.write_sample(s)?;
                }
                return Ok(());
            }
            BitDepth::Int16 => 32_767.0,
            BitDepth::Int24 => 8_388_607.0,
        };
        for &s in interleaved {
            let x = f64::from(s.clamp(-1.0, 1.0)) * full;
            let v = match &mut self.rng {
                None => x.round(),
                Some(state) => (x + uniform(state) + uniform(state) - 1.0)
                    .round()
                    .clamp(-full - 1.0, full),
            };
            // NaN clamps to NaN and converts to 0.
            self.writer.write_sample(v as i32)?;
        }
        Ok(())
    }

    pub fn finalize(self) -> Result<(), hound::Error> {
        self.writer.finalize()
    }
}

/// A uniform value in `0..1` (xorshift64*).
fn uniform(state: &mut u64) -> f64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tpdf_dither_keeps_signal_below_one_lsb_and_is_reproducible() {
        let dir = tempfile::tempdir().unwrap();
        let lsb = 1.0 / 32_767.0;
        // A sine of a quarter LSB: plain rounding writes digital silence.
        let n = 48_000;
        let data: Vec<f32> = (0..2 * n)
            .map(|i| 0.25 * lsb * ((i / 2) as f32 * 1000.0 * std::f32::consts::TAU / 48_000.0).sin())
            .collect();
        let read = |name: &str, dither: Option<u64>| {
            let p = dir.path().join(name);
            write_wav_dithered(&p, 48_000, &data, BitDepth::Int16, dither).unwrap();
            let mut r = hound::WavReader::open(&p).unwrap();
            r.samples::<i32>().map(|v| v.unwrap()).collect::<Vec<_>>()
        };
        assert!(read("plain.wav", None).iter().all(|&v| v == 0));
        let a = read("a.wav", Some(7));
        assert_eq!(a, read("b.wav", Some(7)), "a seed gives the same file");
        assert_ne!(a, read("c.wav", Some(8)));
        assert!(a.iter().all(|v| v.abs() <= 2), "TPDF stays within ±1 LSB of the signal");
        // The signal survives in the noise: correlate the left channel with the sine.
        let corr: f64 = a
            .iter()
            .step_by(2)
            .enumerate()
            .map(|(i, &v)| f64::from(v) * (i as f64 * 1000.0 * std::f64::consts::TAU / 48_000.0).sin())
            .sum::<f64>()
            / (n as f64 / 2.0);
        assert!((corr - 0.25).abs() < 0.05, "{corr}");
        let mean = a.iter().map(|&v| f64::from(v)).sum::<f64>() / a.len() as f64;
        assert!(mean.abs() < 0.02, "unbiased: {mean}");
        // Full scale stays in range, and float output is untouched.
        write_wav_dithered(
            &dir.path().join("loud.wav"),
            48_000,
            &[1.0, -1.0, 2.0, f32::NAN],
            BitDepth::Int24,
            Some(1),
        )
        .unwrap();
        let p = dir.path().join("float.wav");
        write_wav_dithered(&p, 48_000, &data, BitDepth::Float32, Some(7)).unwrap();
        assert_eq!(load_wav(&p).unwrap().data, data);
    }

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
