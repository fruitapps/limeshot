//! Deterministic audio tests: the Channel Rack → sequencer → engine path, the Piano Roll →
//! synth path, the Playlist → song path and mixer routing, verified on rendered samples.

use limeshot_core::*;
use limeshot_dsp::SampleData;
use limeshot_engine::*;
use limeshot_sequencer::NoSamples;
mod support;
use std::sync::Arc;
use support::{Renderer, compile, fx, render_offline};

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

fn snapshot(f: &Fixture) -> Arc<support::Snapshot> {
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

fn render(snap: Arc<support::Snapshot>, range: ExportRange, block: usize, sr: u32) -> (Vec<f32>, RenderReport) {
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
fn plugin_polyphony_does_not_use_the_host_preview_voice_pool() {
    let mut f = fixture();
    for k in 20..100 {
        f.doc
            .execute(Command::AddNote {
                pattern: f.pat,
                note: NewNote {
                    channel: f.synth,
                    key: k,
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
    assert!(buf.iter().any(|v| *v != 0.0));
    assert_eq!(r.active_voices(), 0, "PLG-028: hosted voices are not reported");
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

#[test]
fn pattern_ignores_the_song_time_selection_at_any_block_size() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let bar = 96_000;
    for block in [1, 64, 480, 1000, MAX_BLOCK] {
        for range in [(960, 1920), (3840, 3 * 3840)] {
            let mut r = Renderer::new(SR, snapshot(&f));
            r.set_mode(PlayMode::Song);
            r.set_song_loop(false);
            r.set_loop_range(Some(range));
            r.set_mode(PlayMode::Pattern);
            r.play();
            let mut audio = Vec::new();
            let mut buf = vec![0.0; 2 * block];
            while audio.len() < 2 * 3 * bar {
                r.render(&mut buf);
                audio.extend_from_slice(&buf);
            }
            assert_eq!(
                onsets(&audio[..2 * 3 * bar], 0.1),
                vec![0, bar, 2 * bar],
                "block {block}, range {range:?}: loops the whole pattern"
            );
            assert!(r.is_playing());
            assert!(r.position() < 3840);
            r.locate(10 * 3840);
            assert_eq!(r.position(), 0, "Pattern locate ignores the selection");
            r.set_mode(PlayMode::Song);
            r.locate(10 * 3840);
            assert_eq!(r.position(), range.0, "Song mode retains the selection");
        }
    }
}

#[test]
fn pattern_loops_the_piano_roll_selection() {
    let mut f = fixture();
    // A hit on step 0 and on step 8 (half a bar); the selection covers the second half bar.
    for step in [0, 8] {
        f.doc
            .execute(Command::SetStep {
                pattern: f.pat,
                channel: f.drum,
                step,
                on: true,
            })
            .unwrap();
    }
    let half = 48_000;
    for block in [1, 64, 480, 1000, MAX_BLOCK] {
        let mut r = Renderer::new(SR, snapshot(&f));
        r.set_mode(PlayMode::Pattern);
        r.set_loop_range(Some((0, 3840)));
        r.set_pattern_loop_range(Some((1920, 3840)));
        r.locate(1920);
        r.play();
        let mut audio = Vec::new();
        let mut buf = vec![0.0; 2 * block];
        while audio.len() < 2 * 3 * half {
            r.render(&mut buf);
            audio.extend_from_slice(&buf);
        }
        assert_eq!(
            onsets(&audio[..2 * 3 * half], 0.1),
            vec![0, half, 2 * half],
            "block {block}: only the selected half bar, looped"
        );
        let pos = r.position();
        assert!((1920..3840).contains(&pos), "{pos}");
        r.locate(10 * 3840);
        assert_eq!(r.position(), 1920, "locating past the selection lands at its start");
        // Song mode keeps its own range; clearing the pattern range loops the whole pattern.
        r.set_mode(PlayMode::Song);
        r.locate(10 * 3840);
        assert_eq!(r.position(), 0);
        r.set_mode(PlayMode::Pattern);
        r.set_pattern_loop_range(None);
        r.locate(3000);
        assert_eq!(r.position(), 3000);
    }
}

#[test]
fn song_loops_the_time_selection_at_any_block_size() {
    let mut f = fixture();
    // Pattern A hits on step 0, pattern B on step 8 (half a bar later).
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let b = PatternId(
        f.doc
            .execute(Command::AddPattern { name: "B".into() })
            .unwrap()
            .0
            .unwrap(),
    );
    f.doc
        .execute(Command::SetStep {
            pattern: b,
            channel: f.drum,
            step: 8,
            on: true,
        })
        .unwrap();
    // B A A B: the selection covers the two A bars.
    for (p, bar) in [(b, 0u64), (f.pat, 1), (f.pat, 2), (b, 3)] {
        f.doc
            .execute(Command::AddClip {
                pattern: p,
                track: 0,
                start: bar * 3840,
                length: None,
            })
            .unwrap();
    }
    let bar = 96_000;
    for block in [1, 64, 480, 1000, MAX_BLOCK] {
        let mut r = Renderer::new(SR, snapshot(&f));
        r.set_mode(PlayMode::Song);
        r.set_song_loop(false);
        r.set_loop_range(Some((3840, 3 * 3840)));
        r.locate(3840);
        r.play();
        let mut audio = Vec::new();
        let mut buf = vec![0.0; 2 * block];
        while audio.len() < 2 * 5 * bar {
            r.render(&mut buf);
            audio.extend_from_slice(&buf);
        }
        assert_eq!(
            onsets(&audio[..2 * 5 * bar], 0.1),
            vec![0, bar, 2 * bar, 3 * bar, 4 * bar],
            "block {block}: only the A bars, looped"
        );
        assert!(r.is_playing(), "the selection loops even with song loop off");
        let pos = r.position();
        assert!((3840..3 * 3840).contains(&pos), "{pos}");
        // Locating past the selection lands at its start; clearing it plays the whole song.
        r.locate(10 * 3840);
        assert_eq!(r.position(), 3840);
        r.set_loop_range(None);
        r.locate(3 * 3840);
        assert_eq!(r.position(), 3 * 3840);
    }
}

/// A 1000-frame ramp sample at the engine rate, for sampler parameter tests.
fn ramp_snapshot(params: SamplerParams) -> Arc<support::Snapshot> {
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

#[test]
fn insert_effects_process_the_track_and_extend_the_tail() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    // A 100 ms, single-echo delay on the drum's insert.
    let id = f
        .doc
        .execute(Command::AddEffect {
            track: 1,
            kind: EffectKind::Delay,
        })
        .unwrap()
        .0
        .unwrap();
    for (index, value) in [(0usize, 100.0f32), (1, 0.0), (3, 18000.0), (4, 0.5)] {
        f.doc
            .execute(Command::SetEffectParam {
                track: 1,
                effect: id,
                index,
                value,
            })
            .unwrap();
    }
    let (audio, _) = render(snapshot(&f), ExportRange::Pattern, 512, SR);
    // Dry click at 0, echo at 4800 frames (100 ms at 48 kHz); the mix halves both.
    let hits: Vec<usize> = audio
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, x)| x[0].abs() > 0.05)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(hits.first(), Some(&0));
    assert!(
        hits.iter().any(|&i| (4799..=4801).contains(&i)),
        "echo at 100 ms: {hits:?}"
    );

    // A reverb on the master keeps the export going after the music ends (hit on the last step).
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 15,
            on: true,
        })
        .unwrap();
    f.doc
        .execute(Command::AddEffect {
            track: 0,
            kind: EffectKind::Reverb,
        })
        .unwrap();
    let (audio, rep) = render(snapshot(&f), ExportRange::Pattern, 512, SR);
    assert!(
        rep.frames > rep.music_frames + SR as usize / 4,
        "tail rendered: {} > {}",
        rep.frames,
        rep.music_frames
    );
    let tail = &audio[2 * rep.music_frames..];
    assert!(tail.iter().any(|x| x.abs() > 1e-3), "reverb tail is audible");
}

#[test]
fn effects_without_processors_are_bypassed() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let before = snapshot(&f);
    f.doc
        .execute(Command::AddEffect {
            track: 1,
            kind: EffectKind::Utility,
        })
        .unwrap();
    // Realtime renderer that never received the processor: the chain is skipped, not a crash.
    let mut r = Renderer::new(SR, before);
    r.set_snapshot(snapshot(&f));
    r.play();
    let mut buf = vec![0.0; 2 * 512];
    r.render(&mut buf);
    assert!(buf[0] > 0.5);
    // Adopting it later and rearranging inserts it.
    let id = f.doc.project().mixer.tracks[1].effects[0].id;
    let mut params = EffectKind::Utility.defaults();
    params[0] = -60.0;
    r.adopt_effect(id, support::fx::build(EffectKind::Utility, SR, &params));
    r.rearrange();
    r.stop();
    r.play();
    r.render(&mut buf);
    // Built at −60 dB, but arranging applies the snapshot's parameters (0 dB).
    assert!(
        (buf[0] - 0.78).abs() < 1e-3,
        "the snapshot's params are applied on arrange: {}",
        buf[0]
    );
    assert!(r.take_retired_effect().is_none());
}

#[test]
fn mixer_routes_through_buses() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let peak = |f: &Fixture| {
        let (a, _) = render(snapshot(f), ExportRange::Pattern, 512, SR);
        a.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    };
    let direct = peak(&f);
    assert!(direct > 0.5);
    // Kick (insert 1) → insert 3 at half level instead of the master; insert 3 → master.
    f.doc
        .execute(Command::SetMixerVolume { track: 3, volume: 1.0 })
        .unwrap();
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 3,
            level: Some(0.5),
        })
        .unwrap();
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 0,
            level: None,
        })
        .unwrap();
    assert!((peak(&f) / direct - 0.5).abs() < 1e-4, "through the bus at half level");
    // Muting the bus silences the kick; the bus's effects process it.
    f.doc.execute(Command::SetMixerMute { track: 3, muted: true }).unwrap();
    assert_eq!(peak(&f), 0.0);
    f.doc.execute(Command::SetMixerMute { track: 3, muted: false }).unwrap();
    // Parallel routing: the master as well as the bus → 1.5× in total.
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 0,
            level: Some(1.0),
        })
        .unwrap();
    assert!((peak(&f) / direct - 1.5).abs() < 1e-4);
    // A track with no routes is silent.
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 0,
            level: None,
        })
        .unwrap();
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 3,
            level: None,
        })
        .unwrap();
    assert_eq!(peak(&f), 0.0);
}

/// MIX-008: a pre-fader send carries the track's signal whatever its fader and pan; a
/// post-fader one follows them. The track's own (post-fader) output still follows the fader.
#[test]
fn pre_fader_sends_ignore_the_fader() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 0,
            on: true,
        })
        .unwrap();
    let peak = |f: &Fixture| {
        let (a, _) = render(snapshot(f), ExportRange::Pattern, 256, SR);
        a.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    };
    let run = |f: &mut Fixture, c: Command| f.doc.execute(c).unwrap();
    let full = peak(&f);
    // Kick (insert 1) sends to insert 3 only, the fader at half.
    run(&mut f, Command::SetMixerVolume { track: 3, volume: 1.0 });
    run(
        &mut f,
        Command::SetRoute {
            from: 1,
            to: 3,
            level: Some(1.0),
        },
    );
    run(
        &mut f,
        Command::SetRoute {
            from: 1,
            to: 0,
            level: None,
        },
    );
    let base = f.doc.project().mixer.tracks[1].volume;
    run(
        &mut f,
        Command::SetMixerVolume {
            track: 1,
            volume: base / 2.0,
        },
    );
    let post = peak(&f);
    assert!((post / full - 0.5).abs() < 1e-3, "post-fader: {post} vs {full}");
    run(
        &mut f,
        Command::SetRoutePre {
            from: 1,
            to: 3,
            pre: true,
        },
    );
    let pre = peak(&f);
    assert!(
        (pre * base / full - 1.0).abs() < 1e-3,
        "pre-fader: the fader no longer counts ({pre})"
    );
    run(&mut f, Command::SetMixerVolume { track: 1, volume: 0.0 });
    assert!((peak(&f) - pre).abs() < 1e-4, "fader down: the send still sounds");
    run(&mut f, Command::SetMixerMute { track: 1, muted: true });
    assert_eq!(peak(&f), 0.0, "a muted track sends nothing");
    assert!(
        f.doc
            .execute(Command::SetRoutePre {
                from: 1,
                to: 5,
                pre: true
            })
            .is_err()
    );
}

/// A one-second ramp whose value encodes the frame index (value = i / 48000), placed as an audio
/// clip at bar 1 beat 3 (tick 1920 = 48 000 frames at 120 BPM), trimmed by a quarter of a second
/// at the start and half a second long.
fn audio_clip_fixture() -> (Document, Arc<support::Snapshot>) {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let clip = ClipId(
        doc.execute(Command::AddAudioClip {
            sample: SampleRef { path: "ramp".into() },
            name: "Ramp".into(),
            track: 0,
            start: 1440,
            length: 1440,
        })
        .unwrap()
        .0
        .unwrap(),
    );
    // Trim the first 480 ticks (0.25 s) by dragging the left edge.
    doc.execute(Command::ResizeClipStart { clip, start: 1920 }).unwrap();
    let c = doc.project().clip(clip).unwrap().clone();
    assert_eq!((c.start, c.offset, c.length), (1920, 480, 960));
    let ch = c.audio_channel().unwrap();
    doc.execute(Command::SetChannelVolume {
        channel: ch,
        volume: 1.0,
    })
    .unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
    let data: Vec<f32> = (0..48_000).flat_map(|i| [i as f32 / 48_000.0; 2]).collect();
    let s = Arc::new(SampleData::from_stereo(SR, data));
    let lookup = move |p: &str| (p == "ramp").then(|| s.clone());
    let pat = doc.project().patterns[0].id;
    let snap = Arc::new(compile(doc.project(), pat, &lookup));
    (doc, snap)
}

#[test]
fn hosted_audio_clips_play_their_source_window() {
    let (_, snap) = audio_clip_fixture();
    assert!(snap.song.events.iter().all(|e| e.clip));
    for block in [1, 64, 333, 1024] {
        let (audio, _) = render(snap.clone(), ExportRange::Song, block, SR);
        assert!(audio[..2 * 48_000].iter().all(|v| *v == 0.0));
        for frame in [48_000, 54_000, 71_999] {
            let expected = (frame - 48_000 + 12_000) as f32 / 48_000.0;
            assert!(
                (audio[2 * frame] - expected).abs() < 1e-5,
                "block {block}, frame {frame}"
            );
        }
        assert!(audio[2 * 72_000..].iter().all(|v| *v == 0.0));
    }
}

#[test]
fn seeking_hosted_audio_clips_chases_the_source() {
    let (_, snap) = audio_clip_fixture();
    for block in [64, 333, 1024] {
        let mut r = Renderer::new(SR, snap.clone());
        r.set_mode(PlayMode::Song);
        r.locate(2400);
        r.play();
        let mut buf = vec![0.0; 2 * block];
        r.render(&mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-5, "block {block}: {}", buf[0]);
    }
}

/// A DC sample (0.5) as an audio clip over bar 1 at 120 BPM (96 000 frames), at unity gains.
fn dc_clip_doc() -> (Document, ChannelId) {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let ch = ChannelId(
        doc.execute(Command::AddChannel {
            name: "DC".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef { path: "dc".into() }),
                one_shot: false,
                release_ms: 0.0,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel: ch,
            key: 60,
            start: 0,
            length: 2 * 3840,
            velocity: 1.0,
        },
    })
    .unwrap();
    doc.execute(Command::AddClip {
        pattern: pat,
        track: 0,
        start: 0,
        length: Some(2 * 3840),
    })
    .unwrap();
    doc.execute(Command::SetChannelVolume {
        channel: ch,
        volume: 1.0,
    })
    .unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
    (doc, ch)
}

fn dc_snapshot(doc: &Document) -> Arc<support::Snapshot> {
    let s = Arc::new(SampleData::from_stereo(SR, vec![0.5; 2 * 4 * 48_000]));
    let lookup = move |p: &str| (p == "dc").then(|| s.clone());
    Arc::new(compile(doc.project(), doc.project().patterns[0].id, &lookup))
}

fn automate(doc: &mut Document, target: AutomationTarget, track: u32, start: Tick, points: &[(Tick, f32)]) {
    let clip = ClipId(
        doc.execute(Command::AddAutomationClip {
            target,
            track,
            start,
            length: 3840,
            value: Some(0.5),
            name: None,
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let a = doc.project().clip(clip).unwrap().automation().unwrap();
    doc.execute(Command::SetAutomationPoints {
        automation: a,
        points: points
            .iter()
            .map(|&(tick, value)| AutomationPoint {
                tick,
                value,
                tension: 0.0,
                mode: Default::default(),
            })
            .collect(),
    })
    .unwrap();
}

#[test]
fn automation_sweeps_channel_pan_during_song_playback() {
    let (mut doc, ch) = dc_clip_doc();
    // Pan from hard left to hard right over bar 1 (2 s).
    automate(
        &mut doc,
        AutomationTarget::ChannelPan(ch),
        1,
        0,
        &[(0, 0.0), (3840, 1.0)],
    );
    for block in [64, 333, 1024] {
        let (audio, _) = render(dc_snapshot(&doc), ExportRange::Song, block, SR);
        for f in [0usize, 24_000, 48_000, 72_000, 95_000] {
            let pan = -1.0 + 2.0 * f as f32 / 96_000.0;
            let (l, r) = (audio[2 * f], audio[2 * f + 1]);
            let want = (0.5 * (1.0 - pan).min(1.0), 0.5 * (1.0 + pan).min(1.0));
            // Automation is evaluated every 64 frames: allow one interval of movement.
            assert!(
                (l - want.0).abs() < 2e-3 && (r - want.1).abs() < 2e-3,
                "frame {f}: {l},{r} vs {want:?}"
            );
        }
        // After the automation clip (bar 2) the last value holds: hard right.
        let f = 120_000;
        assert!(
            audio[2 * f] < 1e-6 && (audio[2 * f + 1] - 0.5).abs() < 1e-6,
            "block {block}"
        );
    }
}

#[test]
fn mixer_automation_holds_between_clips_and_only_runs_in_song_mode() {
    let (mut doc, _) = dc_clip_doc();
    // Insert 1 at half travel (gain 0.125 × 2.0 = 0.25) from bar 2 on; before its clip the lane
    // takes its first value too.
    automate(&mut doc, AutomationTarget::MixerVolume(1), 1, 3840, &[(0, 0.5)]);
    let snap = dc_snapshot(&doc);
    let (audio, _) = render(snap.clone(), ExportRange::Song, 512, SR);
    assert!((audio[2 * 1000] - 0.125).abs() < 1e-6, "{}", audio[2 * 1000]);
    assert!((audio[2 * 100_000] - 0.125).abs() < 1e-6);
    // Pattern mode plays the mixer at its set value.
    let mut r = Renderer::new(SR, snap);
    assert_eq!(r.track_mix(1), Some((1.0, 0.0)));
    r.set_mode(PlayMode::Song);
    let mut buf = vec![0.0; 2 * 64];
    r.render(&mut buf);
    assert_eq!(r.track_mix(1), Some((0.25, 0.0)));
    r.set_mode(PlayMode::Pattern);
    r.render(&mut buf);
    assert_eq!(r.track_mix(1), Some((1.0, 0.0)));
}

#[test]
fn automation_drives_effect_parameters() {
    let (mut doc, _) = dc_clip_doc();
    let fx = doc
        .execute(Command::AddEffect {
            track: 1,
            kind: EffectKind::Utility,
        })
        .unwrap()
        .0
        .unwrap();
    // Utility gain spans −24…+24 dB: 0.75 of the range is +12 dB.
    automate(
        &mut doc,
        AutomationTarget::EffectParam {
            track: 1,
            effect: fx,
            index: 0,
        },
        1,
        0,
        &[(0, 0.75)],
    );
    let (audio, _) = render(dc_snapshot(&doc), ExportRange::Song, 256, SR);
    let want = 0.5 * 10f32.powf(12.0 / 20.0);
    // Well after any parameter smoothing has settled.
    assert!(
        (audio[2 * 48_000] - want).abs() < 1e-3,
        "{} vs {want}",
        audio[2 * 48_000]
    );
}

/// A two-beat 220 Hz saw on the fixture's synth with `params`, rendered in Pattern mode.
fn synth_note(params: SynthParams) -> Vec<f32> {
    synth_key(params, 57)
}

/// [`synth_note`] at MIDI key `key`.
fn synth_key(params: SynthParams, key: u8) -> Vec<f32> {
    let mut f = fixture();
    f.doc
        .execute(Command::SetInstrument {
            channel: f.synth,
            instrument: Instrument::Synth(SynthParams {
                attack_ms: 0.0,
                sustain: 1.0,
                release_ms: 5.0,
                ..params
            }),
        })
        .unwrap();
    f.doc
        .execute(Command::AddNote {
            pattern: f.pat,
            note: NewNote {
                channel: f.synth,
                key,
                start: 0,
                length: 1920,
                velocity: 1.0,
            },
        })
        .unwrap();
    render(snapshot(&f), ExportRange::Pattern, 256, SR).0
}

/// RMS of the first difference (rises with high-frequency content) over frames `a..b`.
fn brightness(audio: &[f32], a: usize, b: usize) -> f32 {
    let l: Vec<f32> = audio[2 * a..2 * b].iter().step_by(2).copied().collect();
    let s: f32 = l.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
    (s / l.len() as f32).sqrt()
}

#[test]
fn synth_filter_darkens_and_its_envelope_opens_it_at_the_note_start() {
    let open = synth_note(SynthParams::default());
    let closed = synth_note(SynthParams {
        cutoff_hz: 400.0,
        ..Default::default()
    });
    assert!(
        brightness(&closed, 10_000, 40_000) < 0.3 * brightness(&open, 10_000, 40_000),
        "a 400 Hz low-pass removes most of a saw's upper harmonics"
    );
    let swept = synth_note(SynthParams {
        cutoff_hz: 200.0,
        filter_env: 6.0,
        filter_decay_ms: 100.0,
        ..Default::default()
    });
    assert!(
        brightness(&swept, 0, 2_000) > 3.0 * brightness(&swept, 30_000, 32_000),
        "bright attack, dark tail"
    );
}

#[test]
fn synth_defaults_bypass_the_filter_and_vibrato_moves_the_pitch() {
    let mods = |p: SynthParams| {
        let s = compile(
            &{
                let mut d = Document::default();
                d.execute(Command::AddChannel {
                    name: "S".into(),
                    instrument: Instrument::Synth(p),
                })
                .unwrap();
                d.project().clone()
            },
            PatternId(0),
            &NoSamples,
        );
        match &s.voices[0].2 {
            limeshot_dsp::voice::VoiceInstrument::Synth { mods, .. } => *mods,
            other => panic!("{other:?}"),
        }
    };
    assert!(!mods(SynthParams::default()).any(), "default synth: no filter, no LFO");
    let plain = synth_note(SynthParams::default());
    let vib = synth_note(SynthParams {
        vibrato_cents: 100.0,
        lfo_rate_hz: 4.0,
        ..Default::default()
    });
    // Zero crossings per 1/8 s window: constant without vibrato, varying with it.
    let crossings = |a: &[f32]| -> Vec<usize> {
        (0..6)
            .map(|w| {
                let s = &a[2 * (6_000 * w)..2 * (6_000 * (w + 1))];
                s.iter()
                    .step_by(2)
                    .collect::<Vec<_>>()
                    .windows(2)
                    .filter(|x| *x[0] < 0.0 && *x[1] >= 0.0)
                    .count()
            })
            .collect()
    };
    let (p, v) = (crossings(&plain), crossings(&vib));
    assert!(p.iter().max().unwrap() - p.iter().min().unwrap() <= 1, "{p:?}");
    assert!(v.iter().max().unwrap() - v.iter().min().unwrap() >= 3, "{v:?}");
}

#[test]
fn legacy_track_eq_is_not_an_in_process_effect() {
    let mut f = fixture();
    f.doc
        .execute(Command::AddNote {
            pattern: f.pat,
            note: NewNote {
                channel: f.synth,
                key: 60,
                start: 0,
                length: 960,
                velocity: 1.0,
            },
        })
        .unwrap();
    let (flat, _) = render(snapshot(&f), ExportRange::Pattern, 256, SR);
    f.doc
        .execute(Command::SetMixerEq {
            track: 2,
            band: 2,
            value: EqBand {
                freq: 100.0,
                gain: -18.0,
                width: 0.27,
            },
        })
        .unwrap();
    let (changed, _) = render(snapshot(&f), ExportRange::Pattern, 256, SR);
    assert_eq!(flat, changed, "PLG-029: legacy track EQ is inactive");
}

#[test]
fn sidechained_compressor_ducks_on_the_key_track() {
    let mut f = fixture();
    // A quiet sustained synth on insert 2, a full-scale click on insert 1 at beat 3.
    f.doc
        .execute(Command::AddNote {
            pattern: f.pat,
            note: NewNote {
                channel: f.synth,
                key: 69,
                start: 0,
                length: 4 * PPQ,
                velocity: 0.1,
            },
        })
        .unwrap();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.drum,
            step: 8,
            on: true,
        })
        .unwrap();
    let fx = f
        .doc
        .execute(Command::AddEffect {
            track: 2,
            kind: EffectKind::Compressor,
        })
        .unwrap()
        .0
        .unwrap();
    let set = |doc: &mut Document, index: usize, value: f32| {
        doc.execute(Command::SetEffectParam {
            track: 2,
            effect: fx,
            index,
            value,
        })
        .unwrap();
    };
    // Threshold just above the synth (so it alone is not compressed), ratio 20, fast attack.
    let window = |doc: &Document| {
        let c = click();
        let lookup = move |p: &str| (p == "click").then(|| c.clone());
        let (a, _) = render(
            Arc::new(compile(doc.project(), f.pat, &lookup)),
            ExportRange::Pattern,
            300,
            SR,
        );
        // 20 ms right after the click (which is a single frame at 1 s), and a later reference.
        let rms = |from: usize, to: usize| {
            let s = &a[2 * from..2 * to];
            (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt()
        };
        (rms(48_010, 48_960), rms(40_000, 47_000))
    };
    let (_, alone) = window(&f.doc);
    let threshold = limeshot_dsp::gain_to_db(alone * 2.0) + 3.0;
    set(&mut f.doc, 0, threshold);
    set(&mut f.doc, 1, 20.0);
    set(&mut f.doc, 2, 0.1);
    set(&mut f.doc, 3, 200.0);
    let (after, before) = window(&f.doc);
    assert!(
        (after / before - 1.0).abs() < 0.1,
        "own input: no ducking ({after} vs {before})"
    );
    // "Sidechain to this track": insert 1 also sends to insert 2 at level 0 (key only).
    f.doc
        .execute(Command::SetRoute {
            from: 1,
            to: 2,
            level: Some(0.0),
        })
        .unwrap();
    set(&mut f.doc, COMPRESSOR_SIDECHAIN, 1.0);
    let (after, before) = window(&f.doc);
    assert!(
        limeshot_dsp::gain_to_db(after / before) < -10.0,
        "the click ducks the synth: {after} vs {before}"
    );
    // A key track that does not route here is ignored.
    set(&mut f.doc, COMPRESSOR_SIDECHAIN, 3.0);
    let (after, before) = window(&f.doc);
    assert!((after / before - 1.0).abs() < 0.1);
    // The renderer reports the gain reduction for metering, block by block.
    set(&mut f.doc, COMPRESSOR_SIDECHAIN, 1.0);
    let c = click();
    let lookup = move |p: &str| (p == "click").then(|| c.clone());
    let snap = Arc::new(compile(f.doc.project(), f.pat, &lookup));
    let mut r = Renderer::new(SR, snap.clone());
    for (id, fx) in fx::build_all(&snap, SR) {
        r.adopt_effect(id, fx);
    }
    r.rearrange();
    r.play();
    let mut buf = vec![0.0; 2 * 480];
    let mut most = 0.0f32;
    for b in 0..120 {
        r.render(&mut buf);
        let gr = r.gain_reductions()[2];
        if b < 99 {
            assert!(gr < 0.5, "block {b}: {gr}");
        }
        most = most.max(gr);
    }
    assert!(most > 10.0, "{most}");
}

#[test]
fn plugin_note_pan_reaches_the_voice() {
    let mut f = fixture();
    let note = NoteId(
        f.doc
            .execute(Command::AddNote {
                pattern: f.pat,
                note: NewNote {
                    channel: f.synth,
                    key: 60,
                    start: 0,
                    length: 960,
                    velocity: 1.0,
                },
            })
            .unwrap()
            .0
            .unwrap(),
    );
    let (before, _) = render(snapshot(&f), ExportRange::Pattern, 256, SR);
    f.doc
        .execute(Command::SetNoteValues {
            pattern: f.pat,
            prop: NoteProp::Pan,
            values: vec![(note, 1.0)],
        })
        .unwrap();
    let (after, _) = render(snapshot(&f), ExportRange::Pattern, 256, SR);
    assert!(before.iter().step_by(2).any(|v| v.abs() > 0.01));
    assert!(after.iter().step_by(2).all(|v| *v == 0.0));
    assert!(after.iter().skip(1).step_by(2).any(|v| v.abs() > 0.01));
}

/// Owner's list item 10: Stop while stopped silences everything (auditions, sample previews,
/// effect tails), and Play starts from silence, so a long reverb does not ring into the song.
#[test]
fn stop_while_stopped_and_play_silence_tails_and_previews() {
    let mut f = fixture();
    f.doc
        .execute(Command::AddEffect {
            track: 0,
            kind: EffectKind::Reverb,
        })
        .unwrap();
    let snap = snapshot(&f);
    let mut r = Renderer::new(SR, snap.clone());
    for (id, fx) in support::fx::build_all(&snap, SR) {
        r.adopt_effect(id, fx);
    }
    r.rearrange();
    let mut buf = vec![0.0; 2 * 512];
    let loud = |b: &[f32]| b.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    // An audition and a sample preview, released: the reverb rings on.
    r.preview_note_on(1, 69, 1.0);
    r.preview_sample(click());
    for _ in 0..20 {
        r.render(&mut buf);
    }
    r.preview_note_off(1, 69);
    for _ in 0..4 {
        r.render(&mut buf);
    }
    assert!(loud(&buf) > 1e-3, "a tail before stopping");
    r.stop();
    for _ in 0..3 {
        r.render(&mut buf);
    }
    assert_eq!(r.active_voices(), 0);
    assert_eq!(loud(&buf), 0.0, "Stop while stopped silences the tail");

    // A held audition is cut when Play starts; the reverb starts empty.
    r.preview_note_on(1, 69, 1.0);
    for _ in 0..20 {
        r.render(&mut buf);
    }
    r.play();
    r.render(&mut buf);
    r.render(&mut buf);
    assert_eq!(
        r.active_voices(),
        0,
        "the audition faded at Play (the pattern is empty)"
    );
    for _ in 0..2 {
        r.render(&mut buf);
    }
    assert!(loud(&buf) < 1e-3, "no tail rings into the song: {}", loud(&buf));
}

/// PL-023: an audio clip's fade-in and fade-out ramp its gain linearly, at any block size and
/// when playback starts inside the fade-in.
#[test]
fn clip_fade_model_clamps_to_the_clip() {
    // Audio clip playback/fades are tracked by PLG-024; retain editing validation independently.
    let (mut doc, _) = audio_clip_fixture();
    let clip = doc.project().playlist.clips[0].id;
    doc.execute(Command::SetClipFades {
        clip,
        fade_in: 99_999,
        fade_out: 500,
    })
    .unwrap();
    let c = doc.project().clip(clip).unwrap();
    assert_eq!((c.fade_in, c.fade_out), (c.length, 0));
}

/// Every built-in effect runs as an insert; FX-006: a gate whose threshold is above the clip
/// silences it, one below lets it through.
#[test]
fn reference_dsp_effects_are_finite_and_gate_threshold_works() {
    let peak = |doc: &Document| {
        let (a, _) = render(dc_snapshot(doc), ExportRange::Song, 256, SR);
        a.iter().fold(0.0f32, |m, v| m.max(v.abs()))
    };
    for kind in EffectKind::ALL {
        let (mut doc, _) = dc_clip_doc();
        doc.execute(Command::AddEffect { track: 1, kind }).unwrap();
        let p = peak(&doc);
        assert!(p.is_finite(), "{kind:?}");
    }
    let (mut doc, _) = dc_clip_doc();
    let open = peak(&doc);
    let gate = doc
        .execute(Command::AddEffect {
            track: 1,
            kind: EffectKind::Gate,
        })
        .unwrap()
        .0
        .unwrap();
    // The DC clip is at about −6 dBFS: a −3 dB threshold keeps the gate shut.
    doc.execute(Command::SetEffectParam {
        track: 1,
        effect: gate,
        index: 0,
        value: -3.0,
    })
    .unwrap();
    assert!(peak(&doc) < 1e-4, "shut");
    doc.execute(Command::SetEffectParam {
        track: 1,
        effect: gate,
        index: 0,
        value: -30.0,
    })
    .unwrap();
    assert!((peak(&doc) - open).abs() < 1e-3, "open");
}

/// INS-007: a looping sample plays its loop for as long as the note lasts (even when one-shot),
/// and the sampler's low-pass filter smooths the attack.
#[test]
fn sampler_loops_and_filters() {
    let v = 0.78;
    let looped = ramp_snapshot(SamplerParams {
        looping: true,
        loop_start: 0.5,
        loop_end: 0.75,
        ..Default::default()
    });
    for block in [64, 512] {
        let (a, _) = render(looped.clone(), ExportRange::Pattern, block, SR);
        // One step at 120 BPM is 6000 frames: the note, not the 1000-frame sample, sets the length.
        let n = sounding_frames(&a);
        assert!((5990..=6010).contains(&n), "block {block}: {n} frames");
        let (lo, hi) = ((0.1 + 500.0 / 2000.0) * v, (0.1 + 750.0 / 2000.0) * v);
        for i in [600usize, 760, 2000, 5900] {
            let x = a[2 * i];
            assert!(x >= lo - 1e-4 && x < hi + 1e-4, "frame {i}: {x} outside the loop");
        }
    }
    let (plain, _) = render(ramp_snapshot(SamplerParams::default()), ExportRange::Pattern, 512, SR);
    let (filtered, _) = render(
        ramp_snapshot(SamplerParams {
            cutoff_hz: 200.0,
            ..Default::default()
        }),
        ExportRange::Pattern,
        512,
        SR,
    );
    assert!(filtered[0] < plain[0] * 0.1, "the filter eases the jump at the start");
    assert!(filtered.iter().all(|x| x.is_finite()));
    assert!(
        (filtered[2 * 900] - plain[2 * 900]).abs() < 0.05,
        "the slow ramp passes"
    );
}

/// EXP-010: exporting the time selection renders exactly that span of the song, with clips that
/// began before it already sounding.
#[test]
fn exporting_a_time_selection_renders_that_span() {
    let (_, snap) = audio_clip_fixture();
    for block in [64, 333] {
        let (sel, report) = render(snap.clone(), ExportRange::Selection { from: 2000, to: 2400 }, block, SR);
        assert_eq!(report.music_frames, 10_000, "block {block}");
        assert!((sel[0] - (12_000.0 + 2000.0) / 48_000.0).abs() < 1e-5);
        assert!(sel[..2 * report.music_frames].iter().all(|v| *v > 0.0));
    }
    let empty = render_offline(
        snap,
        &RenderSettings {
            sample_rate: SR,
            range: ExportRange::Selection { from: 100, to: 100 },
            max_tail_secs: 0.0,
            block: 256,
        },
    );
    assert!(empty.is_err(), "an empty selection is an error");
}

#[test]
fn exporting_a_selection_past_the_song_end_is_bounded() {
    let (_, snap) = audio_clip_fixture();
    for block in [1, 64, 333, 1024] {
        let (_, full) = render(snap.clone(), ExportRange::Song, block, SR);
        let (_, selection) = render(
            snap.clone(),
            ExportRange::Selection {
                from: 0,
                to: snap.song.length + 3840,
            },
            block,
            SR,
        );
        assert_eq!(selection.music_frames, full.music_frames);
        assert!(
            render_offline(
                snap.clone(),
                &RenderSettings {
                    sample_rate: SR,
                    range: ExportRange::Selection {
                        from: snap.song.length + 1,
                        to: snap.song.length + 3840
                    },
                    max_tail_secs: 0.0,
                    block
                }
            )
            .is_err()
        );
    }
}

/// INS-014: the sampler's volume envelope: an attack fades the start in, a decay falls to the
/// sustain level.
#[test]
fn sampler_volume_envelope() {
    for block in [1, 64, 333, 1024] {
        let v = 0.78;
        let (a, _) = render(
            ramp_snapshot(SamplerParams {
                attack_ms: 5.0,
                ..Default::default()
            }),
            ExportRange::Pattern,
            block,
            SR,
        );
        // 5 ms is 240 frames at 48 kHz.
        assert!(a[0].abs() < 0.01, "starts silent: {}", a[0]);
        let full = |i: usize| (0.1 + i as f32 / 2000.0) * v;
        assert!(
            a[2 * 120] > 0.2 * full(120) && a[2 * 120] < 0.8 * full(120),
            "half-way up: {}",
            a[2 * 120]
        );
        assert!(
            (a[2 * 400] - full(400)).abs() < 0.01 * full(400),
            "fully open after the attack"
        );
        let (b, _) = render(
            ramp_snapshot(SamplerParams {
                decay_ms: 2.0,
                sustain: 0.5,
                ..Default::default()
            }),
            ExportRange::Pattern,
            block,
            SR,
        );
        assert!((b[0] - full(0)).abs() < 0.01, "no attack");
        assert!(
            (b[2 * 900] - 0.5 * full(900)).abs() < 0.02 * full(900),
            "at the sustain level: {}",
            b[2 * 900]
        );
    }
}

#[test]
fn skipped_sampler_advances_envelope_in_output_time() {
    use limeshot_dsp::voice::{Source, Voice};
    for reverse in [false, true] {
        let snap = ramp_snapshot(SamplerParams {
            attack_ms: 4.0,
            decay_ms: 4.0,
            sustain: 0.5,
            pitch: 12.0,
            reverse,
            ..Default::default()
        });
        let instrument = &snap.voices[0].2;
        // The sample moves two source frames per output frame at this pitch.
        for offset in [0, 96, 192, 288, 384, 480] {
            let mut skipped = Voice::for_instrument(instrument, 60, 60, 1.0, SR as f32).unwrap();
            let mut rendered = skipped.clone();
            for _ in 0..offset {
                rendered.next_frame();
            }
            skipped.skip(f64::from(offset) / f64::from(SR), SR as f32);
            assert!((skipped.env.level() - rendered.env.level()).abs() < 1e-5);
            for _ in 0..8 {
                let (actual, _) = skipped.next_frame();
                let (expected, _) = rendered.next_frame();
                assert!((actual - expected).abs() < 1e-5, "offset {offset}, reverse {reverse}");
            }
            skipped.skip(1.0, SR as f32);
            assert!(matches!(skipped.source, Source::Off));
        }
    }
}

/// AUT-008: a tempo automation re-times the transport during Song playback; the export runs
/// to the song's end at the automated tempo. Bar 1 holds 120 BPM, then the clip ends at 180 BPM,
/// which holds for bar 2.
#[test]
fn tempo_automation_changes_the_song_length() {
    let (mut doc, _) = dc_clip_doc();
    let clip = ClipId(
        doc.execute(Command::AddAutomationClip {
            target: AutomationTarget::Tempo,
            track: 3,
            start: 0,
            length: 3840,
            value: None,
            name: None,
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let a = doc.project().clip(clip).unwrap().automation().unwrap();
    // 60–180 BPM: 0.5 is 120 BPM, 1.0 is 180 BPM.
    doc.execute(Command::SetAutomationPoints {
        automation: a,
        points: vec![
            AutomationPoint {
                tick: 0,
                value: 0.5,
                tension: 0.0,
                mode: PointMode::Hold,
            },
            AutomationPoint {
                tick: 3840,
                value: 1.0,
                tension: 0.0,
                mode: PointMode::SingleCurve,
            },
        ],
    })
    .unwrap();
    let snap = dc_snapshot(&doc);
    for block in [64, 512] {
        let (_, report) = render(snap.clone(), ExportRange::Song, block, SR);
        // Bar 1: 2 s at 120 BPM; bar 2: 4/3 s at 180 BPM.
        let want = 96_000 + 64_000;
        assert!(
            report.music_frames.abs_diff(want) <= 64,
            "block {block}: {} frames, want {want}",
            report.music_frames
        );
    }
    // Live: during bar 2 the renderer plays at 180 BPM; in Pattern mode the project tempo holds.
    let mut r = Renderer::new(SR, snap.clone());
    r.set_mode(PlayMode::Song);
    r.locate(3840 + 100);
    r.play();
    let mut buf = vec![0.0; 2 * 256];
    r.render(&mut buf);
    assert_eq!(r.current_tempo(), Tempo::from_bpm(180.0));
    r.set_mode(PlayMode::Pattern);
    r.render(&mut buf);
    assert_eq!(r.current_tempo(), Tempo::from_bpm(120.0));
    // The project tempo itself is untouched.
    assert_eq!(doc.project().tempo, Tempo::from_bpm(120.0));
}

fn rms_of(audio: &[f32], a: usize, b: usize) -> f32 {
    let l = &audio[2 * a..2 * b];
    (l.iter().map(|x| x * x).sum::<f32>() / l.len() as f32).sqrt()
}

/// INS-013: LimeSynth's high-pass removes a low note's body and keeps a high note's; the
/// band-pass sits between; key tracking opens a low-pass for higher notes.
#[test]
fn synth_filter_modes_and_key_tracking() {
    let hp = SynthParams {
        filter_mode: FilterMode::HighPass,
        cutoff_hz: 2000.0,
        ..Default::default()
    };
    let low = rms_of(&synth_key(hp, 33), 10_000, 40_000);
    let open_low = rms_of(&synth_key(SynthParams::default(), 33), 10_000, 40_000);
    assert!(
        low < 0.35 * open_low,
        "a 2 kHz high-pass thins a 55 Hz saw: {low} vs {open_low}"
    );
    let bp = synth_note(SynthParams {
        filter_mode: FilterMode::BandPass,
        cutoff_hz: 1000.0,
        ..Default::default()
    });
    assert!(bp.iter().all(|x| x.is_finite()) && rms_of(&bp, 10_000, 40_000) > 0.0);
    // A 300 Hz low-pass on a note two octaves above C5: with full key tracking the cutoff
    // follows to about 1.2 kHz and the note is brighter.
    let lp = |track| SynthParams {
        cutoff_hz: 300.0,
        key_track: track,
        ..Default::default()
    };
    let fixed = brightness(&synth_key(lp(0.0), 84), 10_000, 40_000);
    let tracked = brightness(&synth_key(lp(1.0), 84), 10_000, 40_000);
    assert!(
        tracked > 2.0 * fixed,
        "key tracking opens the filter: {tracked} vs {fixed}"
    );
}

#[test]
fn playlist_clip_audition_uses_the_source_offset_at_every_block_size() {
    let (_, snap) = audio_clip_fixture();
    for block in [1, 64, 333, 1024] {
        let mut renderer = Renderer::new(SR, snap.clone());
        let channel = snap.song.events[0].channel;
        renderer.preview_clip(
            channel,
            limeshot_dsp::ClipPlayback {
                offset_secs: 0.25,
                elapsed_secs: 0.125,
                remaining_secs: 0.1,
                fade_in_secs: 0.0,
                fade_out_secs: 0.0,
                fade_in_curve: 0.0,
                fade_out_curve: 0.0,
            },
        );
        let mut out = vec![0.0; block * 2];
        renderer.render(&mut out);
        assert!((out[0] - 0.375).abs() < 1e-5, "block {block}: {}", out[0]);
        renderer.preview_note_off(channel, 60);
        for _ in 0..(1024 / block + 2) {
            renderer.render(&mut out);
        }
        assert!(out.iter().all(|v| v.abs() < 1e-6));
    }
}

/// REC-003: input is audible while stopped and uses the same insert/fader/pan/mute path.
#[test]
fn live_input_runs_through_mixer_effects_fader_pan_routes_and_mute() {
    let mut doc = Document::default();
    doc.execute(Command::SetMixerVolume { track: 0, volume: 1.0 }).unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 0.5 }).unwrap();
    doc.execute(Command::SetMixerPan { track: 1, pan: 0.5 }).unwrap();
    let id = doc
        .execute(Command::AddEffect {
            track: 1,
            kind: EffectKind::Utility,
        })
        .unwrap()
        .0
        .unwrap();
    doc.execute(Command::SetEffectParam {
        track: 1,
        effect: id,
        index: 2,
        value: 1.0,
    })
    .unwrap();
    let pat = doc.project().patterns[0].id;
    for block in [1, 64, 333, 1024] {
        let mut s = compile(doc.project(), pat, &NoSamples);
        // Inject a reference processor into the production insert path; Document normalizes
        // legacy Utility commands to a plugin, whose real host is tested separately.
        s.inner.mixer[1].effects[0].params = vec![0.0, 1.0, 1.0];
        let mut r = Renderer::new(SR, Arc::new(s));
        r.adopt_effect(id, fx::build(EffectKind::Utility, SR, &[0.0, 1.0, 1.0]));
        r.rearrange();
        let input = vec![0.5; block * 2];
        let mut out = vec![0.0; block * 2];
        r.render_with_input(&mut out, Some((1, &input)));
        for frame in out.as_chunks::<2>().0 {
            assert!((frame[0] + 0.125).abs() < 1e-6);
            assert!((frame[1] + 0.25).abs() < 1e-6);
        }
        assert_eq!(r.position(), 0);
        assert!(!r.is_playing());
        assert!(r.peaks()[1].right > 0.24);
        // A regular render (including an export) has no input or retained input samples.
        r.render(&mut out);
        assert!(out.iter().all(|s| *s == 0.0));
        r.render_with_input(&mut out, Some((127, &input)));
        assert!(out.iter().all(|s| *s == 0.0));
    }
    doc.execute(Command::SetMixerMute { track: 1, muted: true }).unwrap();
    let mut r = Renderer::new(SR, Arc::new(compile(doc.project(), pat, &NoSamples)));
    let mut out = [9.0; 128];
    r.render_with_input(&mut out, Some((1, &[0.5; 128])));
    assert!(out.iter().all(|s| *s == 0.0));
}

/// REC-007: the lead-in freezes the musical position and starts the same program sample at
/// exactly the requested bar boundary, even when that boundary splits a render block.
#[test]
fn count_in_is_sample_exact_across_blocks_tempos_and_time_signatures() {
    const RATE: u32 = 8000;
    for (bpm, sig, bars) in [
        (137.123, TimeSignature::new(4, 4), 1),
        (222.0, TimeSignature::new(7, 8), 2),
    ] {
        let mut f = fixture();
        f.doc.execute(Command::SetTempo(Tempo::from_bpm(bpm))).unwrap();
        f.doc.execute(Command::SetTimeSignature(sig)).unwrap();
        f.doc
            .execute(Command::SetStep {
                pattern: f.pat,
                channel: f.drum,
                step: 0,
                on: true,
            })
            .unwrap();
        let snapshot = snapshot(&f);
        let length =
            Timeline::new(f.doc.project().tempo, RATE).sample_of_tick(sig.ticks_per_bar() * bars as u64) as usize;
        let mut reference = Renderer::new(RATE, snapshot.clone());
        reference.play();
        let mut expected = vec![0.0; 1000];
        reference.render(&mut expected);
        for block in [1, 64, 333, 1024] {
            let mut r = Renderer::new(RATE, snapshot.clone());
            r.play_with_count_in(bars);
            assert_eq!(r.count_in_beats(), u32::from(sig.numerator) * u32::from(bars));
            let mut audio = vec![0.0; (length + 500) * 2];
            for (i, chunk) in audio.chunks_mut(block * 2).enumerate() {
                r.render(chunk);
                if (i + 1) * block < length {
                    assert_eq!(r.position(), 0);
                    assert!(r.count_in_beats() > 0);
                }
            }
            assert_eq!(r.count_in_beats(), 0);
            assert_eq!(&audio[2 * length..], &expected, "{bpm} {sig} {bars} bars block {block}");
            assert!(
                audio[..2 * length].iter().any(|s| s.abs() > 0.01),
                "clicks must sound with metronome disabled"
            );
        }
    }
}

#[test]
fn stopping_pausing_and_locating_cancel_count_in() {
    for action in 0..3 {
        let mut r = Renderer::new(SR, Arc::new(compile(&Project::new(), PatternId(1), &NoSamples)));
        r.locate(240);
        r.play_with_count_in(2);
        let mut out = [0.0; 128];
        r.render(&mut out);
        assert_eq!(r.position(), 240);
        match action {
            0 => r.stop(),
            1 => r.pause(),
            _ => r.locate(480),
        }
        assert_eq!(r.count_in_beats(), 0);
        if action < 2 {
            // The existing silence path declicks over 5 ms.
            for _ in 0..8 {
                r.render(&mut out);
            }
            assert!(out.iter().all(|s| *s == 0.0));
            assert_eq!(r.position(), 240);
            r.play();
            assert_eq!(r.count_in_beats(), 0);
        } else {
            assert_eq!(r.position(), 480);
        }
    }
}

#[test]
fn mp3_and_wav_share_range_tail_and_render_report() {
    let mut f = fixture();
    f.doc
        .execute(Command::SetStep {
            pattern: f.pat,
            channel: f.synth,
            step: 15,
            on: true,
        })
        .unwrap();
    f.doc
        .execute(Command::AddClip {
            pattern: f.pat,
            track: 0,
            start: 0,
            length: None,
        })
        .unwrap();
    let snap = snapshot(&f);
    let dir = tempfile::tempdir().unwrap();
    for range in [
        ExportRange::Song,
        ExportRange::Pattern,
        ExportRange::Selection {
            from: 2 * PPQ,
            to: 4 * PPQ,
        },
    ] {
        let settings = RenderSettings {
            sample_rate: 44100,
            range,
            ..Default::default()
        };
        let write = |format, path: &std::path::Path| {
            export_audio_with(
                Arc::new(snap.inner.clone()),
                &settings,
                format,
                path,
                support::processors(&snap, settings.sample_rate),
            )
            .unwrap()
        };
        let wav = dir.path().join("mix.wav");
        let mp3 = dir.path().join("mix.mp3");
        let wav_report = write(ExportFormat::Wav(BitDepth::Float32), &wav);
        let mp3_report = write(ExportFormat::Mp3 { bitrate_kbps: 192 }, &mp3);
        assert_eq!(mp3_report, wav_report);
        assert!(mp3_report.frames > mp3_report.music_frames, "release tail included");
        let decoded = limeshot_dsp::load_audio(&mp3).unwrap();
        assert!(decoded.peak() > 0.01);
        assert!(decoded.frames().abs_diff(mp3_report.frames) < 3456);
    }
}

#[test]
fn browser_scrub_offset_and_replacement_are_independent_of_block_size() {
    let f = fixture();
    let snap = snapshot(&f);
    let sample = Arc::new(SampleData::from_stereo(
        SR,
        (0..8192).map(|i| (i / 2) as f32 / 8192.0).collect(),
    ));
    for block in [1, 64, 333, 1024] {
        let mut full = Renderer::new(SR, snap.clone());
        full.preview_sample(sample.clone());
        let mut reference = vec![0.0; 8192];
        for out in reference.chunks_mut(block * 2) {
            full.render(out);
        }
        let mut scrub = Renderer::new(SR, snap.clone());
        scrub.preview_sample(sample.clone());
        scrub.render(&mut [0.0; 200]);
        scrub.preview_sample_from(sample.clone(), 0.5);
        let mut audio = vec![0.0; 2048];
        for out in audio.chunks_mut(block * 2) {
            scrub.render(out);
        }
        assert_eq!(
            audio[1024..],
            reference[5120..6144],
            "block {block}: source offset after attack and the previous preview release"
        );
        scrub.stop_sample_preview();
        scrub.render(&mut [0.0; 2048]);
        let mut silent = [0.0; 128];
        scrub.render(&mut silent);
        assert!(silent.iter().all(|x| *x == 0.0));
    }
}
