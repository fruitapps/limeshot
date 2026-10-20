//! Deterministic audio tests: the Channel Rack → sequencer → engine path, the Piano Roll →
//! synth path, the Playlist → song path and mixer routing, verified on rendered samples.

use limeshot_core::*;
use limeshot_dsp::SampleData;
use limeshot_engine::*;
use limeshot_sequencer::{NoSamples, compile};
use std::sync::Arc;

const SR: u32 = 48_000;

/// A click sample: one full-scale frame, then silence. Onsets are exact to the sample.
fn click() -> Arc<SampleData> {
    let mut d = vec![0.0f32; 2 * 2000];
    d[0] = 1.0;
    d[1] = 1.0;
    Arc::new(SampleData::from_stereo(SR, d))
}

struct Fixture {
    doc: Document,
    pat: PatternId,
    drum: ChannelId,
    synth: ChannelId,
}

fn fixture() -> Fixture {
    let mut doc = Document::default();
    // The timings below are written for 120 BPM (new projects start at 140).
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let drum = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Click".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef { path: "click".into() }),
                release_ms: 0.0,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let synth = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams {
                attack_ms: 0.0,
                release_ms: 20.0,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    // Unity gains so sample values are easy to reason about.
    for (c, t) in [(drum, 1u16), (synth, 2)] {
        doc.execute(Command::SetChannelVolume {
            channel: c,
            volume: 1.0,
        })
        .unwrap();
        doc.execute(Command::SetMixerVolume { track: t, volume: 1.0 }).unwrap();
    }
    Fixture { doc, pat, drum, synth }
}

fn snapshot(f: &Fixture) -> Arc<limeshot_sequencer::Snapshot> {
    let c = click();
    let lookup = move |p: &str| (p == "click").then(|| c.clone());
    Arc::new(compile(f.doc.project(), f.pat, &lookup))
}

fn onsets(audio: &[f32], threshold: f32) -> Vec<usize> {
    let mut v = Vec::new();
    let mut prev = 0.0f32;
    for (i, fr) in audio.as_chunks::<2>().0.iter().enumerate() {
        let a = fr[0].abs();
        if a > threshold && prev <= threshold {
            v.push(i);
        }
        prev = a;
    }
    v
}

fn render(
    snap: Arc<limeshot_sequencer::Snapshot>,
    range: ExportRange,
    block: usize,
    sr: u32,
) -> (Vec<f32>, RenderReport) {
    render_offline(
        snap,
        &RenderSettings {
            sample_rate: sr,
            range,
            max_tail_secs: 1.0,
            block,
        },
    )
    .unwrap()
}

#[test]
fn steps_sound_exactly_on_the_grid() {
    let mut f = fixture();
    for s in [0, 4, 8, 12] {
        f.doc
            .execute(Command::SetStep {
                pattern: f.pat,
                channel: f.drum,
                step: s,
                on: true,
            })
            .unwrap();
    }
    // 120 BPM, 48 kHz: one beat = 24000 frames.
    let (audio, rep) = render(snapshot(&f), ExportRange::Pattern, 512, SR);
    assert_eq!(rep.music_frames, 96_000);
    assert_eq!(onsets(&audio, 0.1), vec![0, 24_000, 48_000, 72_000]);
    // Only the click frames are non-zero (attack envelope starts at 0 so the very first frame is
    // 0; the sampler's 1 ms attack makes the click quieter but still present).
    assert!(rep.peak > 0.0);
}

#[test]
fn tempo_change_moves_onsets() {
    let mut f = fixture();
    for s in [0, 4] {
        f.doc
            .execute(Command::SetStep {
                pattern: f.pat,
                channel: f.drum,
                step: s,
                on: true,
            })
            .unwrap();
    }
    f.doc.execute(Command::SetTempo(Tempo::from_bpm(90.0))).unwrap();
    let (audio, rep) = render(snapshot(&f), ExportRange::Pattern, 512, SR);
    // One beat at 90 BPM = 32000 frames.
    assert_eq!(onsets(&audio, 0.1), vec![0, 32_000]);
    assert_eq!(rep.music_frames, 128_000);
}

#[test]
fn block_size_does_not_change_the_output() {
    let mut f = fixture();
    for s in [0, 3, 6, 9, 13] {
        f.doc
            .execute(Command::SetStep {
                pattern: f.pat,
                channel: f.drum,
                step: s,
                on: true,
            })
            .unwrap();
    }
    for (k, start) in [(60u8, 0u64), (64, 700), (67, 1777)] {
        f.doc
            .execute(Command::AddNote {
                pattern: f.pat,
                note: NewNote {
                    channel: f.synth,
                    key: k,
                    start,
                    length: 500,
                    velocity: 0.8,
                },
            })
            .unwrap();
    }
    f.doc
        .execute(Command::SetTempo(Tempo::from_milli_bpm(133_333)))
        .unwrap();
    let snap = snapshot(&f);
    let (a, _) = render(snap.clone(), ExportRange::Pattern, 512, 44_100);
    for block in [1, 64, 77, 1000, 1024] {
        let (b, _) = render(snap.clone(), ExportRange::Pattern, block, 44_100);
        assert_eq!(a.len(), b.len(), "block {block}");
        let max_diff = a.iter().zip(&b).fold(0.0f32, |m, (x, y)| m.max((x - y).abs()));
        assert!(max_diff < 1e-6, "block {block}: {max_diff}");
    }
}

#[test]
fn onsets_scale_with_sample_rate() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 1,
            on: true,
        })
        .unwrap();
    for sr in [44_100u32, 48_000, 96_000] {
        let c = Arc::new(SampleData::from_stereo(sr, {
            let mut d = vec![0.0; 4000];
            d[0] = 1.0;
            d[1] = 1.0;
            d
        }));
        let lookup = move |p: &str| (p == "click").then(|| c.clone());
        let snap = Arc::new(compile(f.doc.project(), f.pat, &lookup));
        let (audio, _) = render(snap, ExportRange::Pattern, 256, sr);
        // Step 1 = a 16th note = 1/8 s at 120 BPM.
        // The first sample at or after the exact onset: 44.1 kHz gives 5512.5, so 5513.
        assert_eq!(onsets(&audio, 0.1), vec![(sr as usize).div_ceil(8)], "sr {sr}");
    }
}

#[test]
fn piano_roll_notes_play_and_release() {
    let mut f = fixture();
    f.doc
        .execute(Command::AddNote {
            pattern: f.pat,
            note: NewNote {
                channel: f.synth,
                key: 69,
                start: 960,
                length: 960,
                velocity: 1.0,
            },
        })
        .unwrap();
    let (audio, _) = render(snapshot(&f), ExportRange::Pattern, 512, SR);
    let rms = |a: usize, b: usize| {
        let s: f32 = audio[2 * a..2 * b].iter().step_by(2).map(|x| x * x).sum();
        (s / (b - a) as f32).sqrt()
    };
    assert_eq!(rms(0, 23_990), 0.0, "silent before the note");
    assert!(rms(24_100, 47_000) > 0.05, "sounding during the note");
    // Released at 48000; 20 ms release = 960 frames.
    assert_eq!(rms(49_500, 95_000), 0.0, "silent after the release");
    // Pitch: count zero crossings of the 440 Hz saw over 0.5 s.
    let mid = &audio[2 * 24_000..2 * 48_000];
    let zc = mid
        .as_chunks::<2>()
        .0
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|w| w[0][0] < 0.0 && w[1][0] >= 0.0)
        .count();
    assert!((218..=222).contains(&zc), "{zc}");
}

#[test]
fn song_mode_plays_the_arrangement() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let p2 = PatternId(
        f.doc
            .execute(Command::AddPattern { name: "B".into() })
            .unwrap()
            .0
            .unwrap(),
    );
    f.doc
        .execute(Command::SetStep {
            pattern: p2,
            channel: f.drum,
            step: 2,
            on: true,
        })
        .unwrap();
    // Bar 1: pattern 1. Bar 3: pattern 1 again (linked). Bar 4: pattern 2.
    for (p, start) in [(f.pat, 0u64), (f.pat, 2 * 3840), (p2, 3 * 3840)] {
        f.doc
            .execute(Command::AddClip {
                pattern: p,
                track: 0,
                start,
                length: None,
            })
            .unwrap();
    }
    let (audio, rep) = render(snapshot(&f), ExportRange::Song, 512, SR);
    let bar = 96_000;
    assert_eq!(rep.music_frames, 4 * bar);
    assert_eq!(onsets(&audio, 0.1), vec![0, 2 * bar, 3 * bar + 12_000]);
}

#[test]
fn empty_song_is_an_error() {
    let f = fixture();
    assert!(matches!(
        render_offline(snapshot(&f), &RenderSettings::default()),
        Err(ExportError::EmptySong)
    ));
}

#[test]
fn mixer_volume_pan_and_mute_route_to_master() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let peak_lr = |doc: &Document, pat| {
        let c = click();
        let lookup = move |p: &str| (p == "click").then(|| c.clone());
        let (a, _) = render(
            Arc::new(compile(doc.project(), pat, &lookup)),
            ExportRange::Pattern,
            512,
            SR,
        );
        let l = a.iter().step_by(2).fold(0.0f32, |m, v| m.max(v.abs()));
        let r = a.iter().skip(1).step_by(2).fold(0.0f32, |m, v| m.max(v.abs()));
        (l, r)
    };
    let (l0, r0) = peak_lr(&f.doc, f.pat);
    assert!(l0 > 0.0 && (l0 - r0).abs() < 1e-6);
    f.doc
        .execute(Command::SetMixerVolume { track: 1, volume: 0.5 })
        .unwrap();
    let (l1, _) = peak_lr(&f.doc, f.pat);
    assert!((l1 / l0 - 0.5).abs() < 1e-4, "insert gain");
    f.doc.execute(Command::SetMixerPan { track: 1, pan: 1.0 }).unwrap();
    let (l2, r2) = peak_lr(&f.doc, f.pat);
    assert_eq!(l2, 0.0);
    assert!((r2 / l0 - 0.5).abs() < 1e-4, "hard right keeps the right side");
    f.doc.execute(Command::SetMixerMute { track: 1, muted: true }).unwrap();
    assert_eq!(peak_lr(&f.doc, f.pat), (0.0, 0.0));
    f.doc.execute(Command::SetMixerMute { track: 1, muted: false }).unwrap();
    f.doc
        .execute(Command::SetChannelMixerTrack {
            channel: f.drum,
            track: 0,
        })
        .unwrap();
    let (l3, r3) = peak_lr(&f.doc, f.pat);
    assert!(
        (l3 - l0).abs() < 1e-6 && (r3 - l0).abs() < 1e-6,
        "routed straight to master"
    );
    f.doc
        .execute(Command::SetChannelMute {
            channel: f.drum,
            muted: true,
        })
        .unwrap();
    assert_eq!(peak_lr(&f.doc, f.pat), (0.0, 0.0));
}

#[test]
fn realtime_transport_loops_pattern_and_reports_position() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let mut r = Renderer::new(SR, snapshot(&f));
    r.play();
    let mut buf = vec![0.0; 2 * 480];
    let mut hits = Vec::new();
    for b in 0..(3 * 96_000 / 480) {
        r.render(&mut buf);
        if buf.as_chunks::<2>().0.iter().any(|x| x[0] > 0.1) {
            hits.push(b * 480);
        }
    }
    assert_eq!(hits, vec![0, 96_000, 192_000], "loops every bar");
    assert!(r.position() < 3840);
    r.stop();
    assert_eq!(r.position(), 0);
    assert!(!r.is_playing());
}

#[test]
fn tempo_change_while_playing_keeps_position() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 8,
            on: true,
        })
        .unwrap();
    let mut r = Renderer::new(SR, snapshot(&f));
    r.play();
    let mut buf = vec![0.0; 2 * 1000];
    for _ in 0..24 {
        r.render(&mut buf); // 24000 frames = 1 beat at 120
    }
    assert_eq!(r.position(), 960);
    f.doc.execute(Command::SetTempo(Tempo::from_bpm(60.0))).unwrap();
    r.set_snapshot(snapshot(&f));
    assert_eq!(r.position(), 960);
    // Step 8 = tick 1920: one more beat, now 48000 frames at 60 BPM.
    let mut first = None;
    for b in 0..60 {
        r.render(&mut buf);
        if first.is_none()
            && let Some(i) = buf.as_chunks::<2>().0.iter().position(|x| x[0] > 0.1)
        {
            first = Some(b * 1000 + i);
        }
    }
    assert_eq!(first, Some(48_000));
}

#[test]
fn preview_plays_without_transport() {
    let f = fixture();
    let mut r = Renderer::new(SR, snapshot(&f));
    r.preview_note_on(1, 69, 1.0);
    let mut buf = vec![0.0; 2 * 512];
    r.render(&mut buf);
    assert!(buf.iter().any(|v| v.abs() > 0.01));
    r.preview_note_off(1, 69);
    for _ in 0..10 {
        r.render(&mut buf);
    }
    assert_eq!(r.active_voices(), 0);
    let _ = NoSamples;
}

#[test]
fn synth_polyphony_and_voice_limit() {
    let mut f = fixture();
    for k in 0..(MAX_VOICES as u8 + 20) {
        f.doc
            .execute(Command::AddNote {
                pattern: f.pat,
                note: NewNote {
                    channel: f.synth,
                    key: 20 + k % 100,
                    start: 0,
                    length: 3000,
                    velocity: 0.1,
                },
            })
            .unwrap();
    }
    let mut r = Renderer::new(SR, snapshot(&f));
    r.play();
    let mut buf = vec![0.0; 2 * 512];
    r.render(&mut buf);
    assert_eq!(
        r.active_voices(),
        MAX_VOICES,
        "excess notes steal voices instead of growing"
    );
}

#[test]
fn song_stops_at_the_end_without_loop() {
    let mut f = fixture();
    f.doc
        .execute(Command::AddClip {
            pattern: f.pat,
            track: 0,
            start: 0,
            length: None,
        })
        .unwrap();
    let mut r = Renderer::new(SR, snapshot(&f));
    r.set_mode(PlayMode::Song);
    r.set_song_loop(false);
    r.play();
    let mut buf = vec![0.0; 2 * 1000];
    for _ in 0..100 {
        r.render(&mut buf);
    }
    assert!(!r.is_playing());
    assert!(r.finished());
    assert_eq!(r.position(), 0);
}

/// A 1000-frame ramp sample at the engine rate, for sampler parameter tests.
fn ramp_snapshot(params: SamplerParams) -> Arc<limeshot_sequencer::Snapshot> {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let ch = ChannelId(
        doc.execute(Command::AddChannel {
            name: "R".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef { path: "ramp".into() }),
                release_ms: 0.0,
                ..params
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::SetChannelVolume {
        channel: ch,
        volume: 1.0,
    })
    .unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
    doc.execute(Command::SetStep {
        pattern: pat,
        channel: ch,
        step: 0,
        on: true,
    })
    .unwrap();
    let data: Vec<f32> = (0..1000).flat_map(|i| [0.1 + i as f32 / 2000.0; 2]).collect();
    let s = Arc::new(SampleData::from_stereo(SR, data));
    let lookup = move |p: &str| (p == "ramp").then(|| s.clone());
    Arc::new(compile(doc.project(), pat, &lookup))
}

fn sounding_frames(audio: &[f32]) -> usize {
    audio
        .as_chunks::<2>()
        .0
        .iter()
        .take_while(|f| f[0].abs() > 1e-6)
        .count()
}

#[test]
fn sampler_pitch_start_end_and_reverse() {
    let (a, _) = render(ramp_snapshot(SamplerParams::default()), ExportRange::Pattern, 512, SR);
    assert_eq!(sounding_frames(&a), 1000, "plays the whole sample once");
    // Steps play at velocity 0.78.
    let v = 0.78;
    assert!((a[0] - 0.1 * v).abs() < 1e-6 && a[2 * 999] > 0.59 * v);

    let (a, _) = render(
        ramp_snapshot(SamplerParams {
            pitch: 12.0,
            ..Default::default()
        }),
        ExportRange::Pattern,
        512,
        SR,
    );
    assert_eq!(sounding_frames(&a), 500, "an octave up plays twice as fast");

    let (a, _) = render(
        ramp_snapshot(SamplerParams {
            start: 0.5,
            end: 0.75,
            ..Default::default()
        }),
        ExportRange::Pattern,
        512,
        SR,
    );
    assert_eq!(sounding_frames(&a), 250, "start/end trim the region");
    assert!(
        (a[0] - (0.1 + 500.0 / 2000.0) * v).abs() < 1e-6,
        "starts at the start point"
    );

    let (a, _) = render(
        ramp_snapshot(SamplerParams {
            reverse: true,
            ..Default::default()
        }),
        ExportRange::Pattern,
        512,
        SR,
    );
    assert_eq!(sounding_frames(&a), 1000);
    assert!(a[0] > a[2 * 500], "reverse plays from the end");

    let (a, _) = render(
        ramp_snapshot(SamplerParams {
            gain_db: -6.0206,
            ..Default::default()
        }),
        ExportRange::Pattern,
        512,
        SR,
    );
    assert!((a[0] - 0.05 * v).abs() < 1e-4, "gain in dB");
}

#[test]
fn metronome_clicks_on_every_beat() {
    let f = fixture();
    let mut r = Renderer::new(SR, snapshot(&f));
    r.set_metronome(true);
    r.play();
    let mut audio = vec![0.0f32; 2 * 96_000];
    for chunk in audio.chunks_mut(2 * 500) {
        r.render(chunk);
    }
    // A click is a short tone: keep the first threshold crossing of each burst.
    let mut on: Vec<usize> = Vec::new();
    for o in onsets(&audio, 0.01) {
        if on.last().is_none_or(|l| o - l > 4_000) {
            on.push(o);
        }
    }
    assert_eq!(on.len(), 4, "{on:?}");
    for (i, o) in on.iter().enumerate() {
        assert!(o.abs_diff(i * 24_000) < 30, "beat {i} at {o}");
    }
}
