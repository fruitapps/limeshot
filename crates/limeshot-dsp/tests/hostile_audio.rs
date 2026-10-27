//! Deterministic corrupt-file probes of the public sample import boundary. Successful
//! decodes may skip damaged compressed frames, but must remain bounded, stereo and finite.

use limeshot_dsp::{BitDepth, SampleData, load_audio_bounded, write_wav};
use std::path::Path;

const FRAME_LIMIT: usize = 16_384;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 as usize
    }
}

fn check_sample(sample: &SampleData, path: &Path) {
    assert!(sample.sample_rate > 0, "{}: zero sample rate", path.display());
    assert!(sample.frames() <= FRAME_LIMIT, "{}: frame limit", path.display());
    assert_eq!(sample.data.len() % 2, 0, "{}: incomplete stereo frame", path.display());
    assert!(
        sample.data.iter().all(|v| v.is_finite()),
        "{}: nonfinite PCM",
        path.display()
    );
    assert_eq!(sample.source.as_deref(), Some(path));
}

#[test]
fn corrupted_audio_bytes_never_panic_or_escape_decode_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let pcm: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
    let mut inputs = Vec::new();
    for depth in BitDepth::ALL {
        let name = format!("{depth:?}.wav");
        let path = dir.path().join(&name);
        write_wav(&path, 22_050, &pcm, depth).unwrap();
        inputs.push((name, std::fs::read(path).unwrap()));
    }
    for name in ["sine.flac", "sine.aiff"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
        inputs.push((name.to_owned(), std::fs::read(path).unwrap()));
    }
    // MPEG-1 Layer III, 128 kbit/s at 44.1 kHz, zero side information and audio data.
    // Generated here so this probe needs no encoder or third-party audio asset.
    let mut frame = vec![0; 417];
    frame[..4].copy_from_slice(&[0xff, 0xfb, 0x90, 0]);
    inputs.push(("silence.mp3".to_owned(), frame.repeat(8)));

    let mut rng = Rng(0x4c69_6d65_5368_6f74);
    for (name, original) in inputs {
        let path = dir.path().join(&name);
        std::fs::write(&path, &original).unwrap();
        // A corrupt-input test must also prove that its original reaches a working decoder.
        check_sample(&load_audio_bounded(&path, FRAME_LIMIT).unwrap(), &path);
        for round in 0..400 {
            let mut bytes = original.clone();
            match round % 4 {
                0 => {
                    for _ in 0..1 + rng.next() % 8 {
                        let index = rng.next() % bytes.len();
                        bytes[index] = rng.next() as u8;
                    }
                }
                1 => bytes.truncate(rng.next() % (bytes.len() + 1)),
                2 => {
                    let index = rng.next() % (bytes.len() - 3);
                    bytes[index..index + 4].fill(0xff);
                }
                _ => {
                    // Concentrate mutations on container lengths and codec configuration.
                    let index = rng.next() % 64.min(bytes.len());
                    bytes[index] = [0, 1, 0x7f, 0x80, 0xff][rng.next() % 5];
                }
            }
            std::fs::write(&path, bytes).unwrap();
            let result = std::panic::catch_unwind(|| load_audio_bounded(&path, FRAME_LIMIT));
            match result {
                Ok(Ok(sample)) => check_sample(&sample, &path),
                Ok(Err(_)) => {}
                Err(_) => panic!("{name}: decoder panicked in mutation round {round}"),
            }
        }
    }
}

#[test]
fn float_wav_import_replaces_nonfinite_pcm_with_silence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonfinite.wav");
    write_wav(
        &path,
        48_000,
        &[f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.5],
        BitDepth::Float32,
    )
    .unwrap();
    let sample = load_audio_bounded(&path, FRAME_LIMIT).unwrap();
    assert_eq!(sample.data, [0.0, 0.0, 0.0, 0.5]);
}

#[test]
fn aiff_zero_sample_rate_returns_a_decode_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("zero-rate.aiff");
    let mut bytes = include_bytes!("fixtures/sine.aiff").to_vec();
    // COMM stores its sample rate as a ten-byte extended float at offset 28.
    bytes[28..38].fill(0);
    std::fs::write(&path, bytes).unwrap();
    assert!(matches!(
        load_audio_bounded(&path, FRAME_LIMIT),
        Err(limeshot_dsp::SampleError::Decode { .. })
    ));
}
