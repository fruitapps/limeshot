//! The MVP acceptance workflow (START.md §13), driven through the same typed commands the UI
//! issues, then verified on the rendered audio:
//! new project → tempo → sampler channel with a WAV drum → drum steps → synth channel → melody
//! notes → more patterns → arrange on the Playlist → song mode → mixer routing, volume, pan →
//! save → reopen → compare → export WAV → inspect the exported file.

use limeshot_audio::assets::SampleCache;
use limeshot_core::*;
use limeshot_dsp::load_wav;
use limeshot_engine::{BitDepth, ExportRange, RenderSettings, export_wav};
use std::sync::Arc;

fn rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
}

/// Left-channel samples of frames `a..b`.
fn left(audio: &[f32], a: usize, b: usize) -> Vec<f32> {
    audio[2 * a..2 * b].iter().step_by(2).copied().collect()
}

#[test]
fn mvp_workflow_end_to_end() {
    let samples = limeshot_app::bundled_samples_dir().expect("bundled samples");
    let dir = tempfile::tempdir().unwrap();

    // 1–3: new project, tempo.
    let mut d = Document::default();
    d.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let drums = d.project().patterns[0].id;

    // 4–6: sampler channel with a WAV kick, four-on-the-floor.
    let kick = ChannelId(
        d.execute(Command::AddChannel {
            name: "Kick".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: samples.join("Lime Kick.wav").to_string_lossy().into_owned(),
                }),
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    for s in [0, 4, 8, 12] {
        d.execute(Command::SetStep {
            pattern: drums,
            channel: kick,
            step: s,
            on: true,
        })
        .unwrap();
    }

    // 7–10: synth channel and a melody in a second pattern.
    let lead = ChannelId(
        d.execute(Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams::default()),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let melody = PatternId(
        d.execute(Command::AddPattern { name: "Melody".into() })
            .unwrap()
            .0
            .unwrap(),
    );
    for (i, k) in [69u8, 72, 76, 72].iter().enumerate() {
        d.execute(Command::AddNote {
            pattern: melody,
            note: NewNote {
                channel: lead,
                key: *k,
                start: i as u64 * PPQ,
                length: PPQ / 2,
                velocity: 0.8,
            },
        })
        .unwrap();
    }

    // 11–14: arrange: drums in bars 1–2, melody in bar 2 only.
    let bar = d.project().time_signature.ticks_per_bar();
    d.execute(Command::AddClip {
        pattern: drums,
        track: 0,
        start: 0,
        length: None,
    })
    .unwrap();
    d.execute(Command::AddClip {
        pattern: drums,
        track: 0,
        start: bar,
        length: None,
    })
    .unwrap();
    d.execute(Command::AddClip {
        pattern: melody,
        track: 1,
        start: bar,
        length: None,
    })
    .unwrap();

    // 15–16: routing, volume and pan.
    assert_eq!(d.project().channel(kick).unwrap().mixer_track, 1);
    assert_eq!(d.project().channel(lead).unwrap().mixer_track, 2);
    d.execute(Command::SetMixerVolume { track: 2, volume: 0.6 }).unwrap();
    d.execute(Command::SetChannelPan {
        channel: lead,
        pan: -0.5,
    })
    .unwrap();

    // 17–19: save, reopen, compare.
    let path = dir.path().join("mvp.lime");
    limeshot_project::save(d.project(), &path).unwrap();
    let reopened = limeshot_project::load(&path).unwrap();
    assert_eq!(&reopened, d.project(), "the composition survives save and reopen");

    // 20: export WAV from the reopened project.
    let mut cache = SampleCache::new();
    cache.set_project_dir(Some(dir.path().to_path_buf()));
    for r in reopened.sample_refs() {
        cache.load_blocking(&r.path).unwrap();
    }
    let snap = Arc::new(limeshot_sequencer::compile(&reopened, drums, &cache));
    let wav = dir.path().join("mvp.wav");
    let settings = RenderSettings {
        sample_rate: 48_000,
        range: ExportRange::Song,
        ..RenderSettings::default()
    };
    let rep = export_wav(snap, &settings, BitDepth::Int24, &wav).unwrap();
    assert_eq!(rep.music_frames, 2 * 96_000, "two bars at 120 BPM");

    // 21: open the exported file and check the musical result.
    let audio = load_wav(&wav).unwrap();
    assert_eq!(audio.sample_rate, 48_000);
    assert!(audio.frames() >= 2 * 96_000);
    assert!(audio.peak() < 1.0, "no clipping");
    // Kick onsets every beat: energy right after each beat is far above the energy just before.
    for beat in 0..8usize {
        let at = beat * 24_000;
        let after = rms(&left(&audio.data, at, at + 2_000));
        let before = if at == 0 {
            0.0
        } else {
            rms(&left(&audio.data, at - 200, at))
        };
        assert!(
            after > 0.05 && after > 4.0 * before,
            "kick at beat {beat}: {after} vs {before}"
        );
    }
    // Bar 1 has no melody; bar 2 does. Compare the right channel between kicks (the lead is
    // panned left, so measure the difference between sides).
    let side = |a: usize, b: usize| -> f32 {
        let d: Vec<f32> = audio.data[2 * a..2 * b]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|f| f[0] - f[1])
            .collect();
        rms(&d)
    };
    assert!(side(12_000, 22_000) < 1e-4, "bar 1 is centred (drums only)");
    assert!(
        side(96_000 + 2_000, 96_000 + 10_000) > 0.01,
        "bar 2 has the left-panned melody"
    );
}

#[test]
fn demo_song_renders_and_reloads_identically() {
    let samples = limeshot_app::bundled_samples_dir().expect("bundled samples");
    let dir = tempfile::tempdir().unwrap();
    let doc = limeshot_app::demo::build(&samples).unwrap();
    let path = dir.path().join("demo.lime");
    limeshot_project::save(doc.project(), &path).unwrap();
    let back = limeshot_project::load(&path).unwrap();
    assert_eq!(&back, doc.project());
    let mut cache = SampleCache::new();
    for r in back.sample_refs() {
        cache.load_blocking(&r.path).unwrap();
    }
    let pat = back.patterns[0].id;
    let (audio, rep) = limeshot_engine::render_offline(
        Arc::new(limeshot_sequencer::compile(&back, pat, &cache)),
        &RenderSettings::default(),
    )
    .unwrap();
    assert!(rep.peak > 0.1 && rep.peak < 1.0, "peak {}", rep.peak);
    assert!(rms(&audio) > 0.01);
}
