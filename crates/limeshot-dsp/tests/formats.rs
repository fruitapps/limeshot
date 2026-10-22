//! INS-008: samples in AIFF and FLAC decode like the WAV they were made from (fixtures: a
//! 0.1 s, 441 Hz stereo sine at 22.05 kHz, made by LimeShot's tests; left at 16000/32768, right
//! at half that). Unknown or damaged files fail with an error, never a panic.

use limeshot_dsp::{is_audio_file, load_audio};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn check(name: &str) {
    let s = load_audio(&fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert_eq!(s.sample_rate, 22_050, "{name}");
    assert_eq!(s.frames(), 2205, "{name}");
    let want = |i: usize| (16000.0 * (2.0 * std::f32::consts::PI * 441.0 * i as f32 / 22_050.0).sin()) as i32;
    for i in [0usize, 7, 12, 1000, 2204] {
        let (l, r) = s.frame(i);
        let wl = want(i) as f32 / 32768.0;
        assert!((l - wl).abs() < 2e-4, "{name} frame {i}: {l} vs {wl}");
        assert!((r - wl / 2.0).abs() < 2e-4, "{name} frame {i} right: {r}");
    }
}

#[test]
fn aiff_and_flac_decode_sample_for_sample() {
    check("sine.aiff");
    check("sine.flac");
}

#[test]
fn audio_extensions_and_bad_files() {
    for ok in ["a.wav", "b.MP3", "c.flac", "d.ogg", "e.aif", "f.AIFF"] {
        assert!(is_audio_file(Path::new(ok)), "{ok}");
    }
    for no in ["a.txt", "b", "c.lime", "d.mid"] {
        assert!(!is_audio_file(Path::new(no)), "{no}");
    }
    let dir = tempfile::tempdir().unwrap();
    for (name, bytes) in [
        ("junk.mp3", &b"ID3 not really an mp3"[..]),
        ("empty.ogg", &b""[..]),
        ("cut.flac", &std::fs::read(fixture("sine.flac")).unwrap()[..60]),
    ] {
        let p = dir.path().join(name);
        std::fs::write(&p, bytes).unwrap();
        assert!(load_audio(&p).is_err(), "{name} is refused");
    }
    assert!(load_audio(&dir.path().join("missing.flac")).is_err());
}
