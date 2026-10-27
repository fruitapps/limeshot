//! The Lime VST3 plugins sound like the former built-in instruments and effects they replace
//! (decision D39): a project rendered with the built-ins and the same project converted to Lime
//! plugins and rendered through the plugin host give the same audio. Needs the bundles
//! (`python3 native-plugins/build.py --debug` and LIMESHOT_LIME_PLUGINS); every test skips with a
//! diagnostic outside CI when required bundles are missing, and fails in CI instead.
use std::sync::Arc;

use limeshot_core::*;
use limeshot_engine::{Processors, RenderSettings, render_offline_with};
use limeshot_sequencer::compile;
use limeshot_ui::plugins::PluginHost;

#[path = "../../limeshot-engine/tests/support/mod.rs"]
mod reference;

const SR: u32 = 48_000;

/// Keep optional local integration tests and mandatory CI bundle validation consistent.
fn bundles_or_skip(names: &[&str]) -> bool {
    let missing: Vec<_> = names
        .iter()
        .filter(|name| limeshot_plugins::lime::bundle_path(name).is_none())
        .collect();
    if missing.is_empty() {
        return true;
    }
    assert!(
        std::env::var_os("CI").is_none(),
        "CI must build required Lime VST3 bundles first: {missing:?}"
    );
    eprintln!(
        "Lime VST3 bundles unavailable: {missing:?}; skipping (run python3 native-plugins/build.py --debug and set LIMESHOT_LIME_PLUGINS)"
    );
    false
}

fn project(wav: &str) -> (Project, PatternId) {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let synth = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams {
                attack_ms: 2.0,
                release_ms: 40.0,
                cutoff_hz: 3000.0,
                resonance: 0.3,
                filter_mode: FilterMode::BandPass,
                key_track: 0.5,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let drum = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Hit".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: std::path::Path::new(wav)
                    .exists()
                    .then(|| SampleRef { path: wav.into() }),
                pitch: 3.0,
                end: 0.8,
                attack_ms: 5.0,
                decay_ms: 80.0,
                sustain: 0.4,
                cutoff_hz: 1500.0,
                resonance: 0.5,
                looping: true,
                loop_start: 0.1,
                loop_end: 0.3,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    let mut notes = [(57, 0), (64, PPQ), (69, 2 * PPQ)]
        .map(|(key, start)| NewNote {
            channel: synth,
            key,
            start,
            length: PPQ / 2,
            velocity: 0.8,
        })
        .to_vec();
    notes.extend([0, PPQ / 2, 3 * PPQ / 2].map(|start| NewNote {
        channel: drum,
        key: 60,
        start,
        length: PPQ / 4,
        velocity: 1.0,
    }));
    doc.execute(Command::AddNotes { pattern: pat, notes }).unwrap();
    let track = doc.project().channel(synth).unwrap().mixer_track;
    doc.execute(Command::AddEffect {
        track,
        kind: EffectKind::Eq3,
    })
    .unwrap();
    let eq = doc.project().mixer.tracks[track as usize].effects[0].id;
    doc.execute(Command::SetEffectParam {
        track,
        effect: eq,
        index: 2,
        value: 6.0,
    })
    .unwrap();
    (doc.project().clone(), pat)
}

fn render(project: &Project, pat: PatternId, processors: Processors) -> Vec<f32> {
    let settings = RenderSettings {
        sample_rate: SR,
        range: limeshot_engine::ExportRange::Pattern,
        max_tail_secs: 0.5,
        block: 256,
    };
    let lookup = |p: &str| limeshot_dsp::load_audio(std::path::Path::new(p)).ok().map(Arc::new);
    let snap = Arc::new(compile(project, pat, &lookup));
    render_offline_with(snap, &settings, processors).unwrap().0
}

#[test]
fn converted_projects_sound_the_same_through_the_lime_vst3s() {
    if !bundles_or_skip(&["lime:LimeSynth.vst3", "lime:Lime Sampler.vst3", "lime:Lime EQ.vst3"]) {
        return;
    }
    // A decaying noise burst, so pitch and trim changes show in the comparison.
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("hit.wav");
    let mut seed = 1u32;
    let burst: Vec<f32> = (0..9600)
        .flat_map(|i| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let v = (seed >> 8) as f32 / 8_388_608.0 - 1.0;
            let g = (-(i as f32) / 2000.0).exp();
            [v * g, -v * g]
        })
        .collect();
    limeshot_dsp::write_wav(&wav, SR, &burst, limeshot_dsp::BitDepth::Float32).unwrap();
    let (built_in, pat) = project(wav.to_str().unwrap());
    let lookup = |p: &str| limeshot_dsp::load_audio(std::path::Path::new(p)).ok().map(Arc::new);
    let reference = Arc::new(reference::compile(&built_in, pat, &lookup));
    let expected = reference::render_offline(
        reference,
        &RenderSettings {
            sample_rate: SR,
            range: limeshot_engine::ExportRange::Pattern,
            max_tail_secs: 0.5,
            block: 256,
        },
    )
    .unwrap()
    .0;

    let mut converted = built_in.clone();
    let warnings = lime::convert_project(&mut converted, &limeshot_ui::lime::embedded_audio);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(!lime::needs_conversion(&converted));
    let host = PluginHost::new(None);
    let (processors, _keep) = host.offline(&converted, SR).unwrap();
    assert_eq!(
        processors.instruments.len(),
        2,
        "LimeSynth and Lime Sampler did not both load"
    );
    assert_eq!(processors.effects.len(), 1, "Lime EQ did not load");
    let actual = render(&converted, pat, processors);

    let rms = |a: &[f32]| (a.iter().map(|v| v * v).sum::<f32>() / a.len().max(1) as f32).sqrt();
    let n = expected.len().min(actual.len());
    let diff: Vec<f32> = expected[..n].iter().zip(&actual[..n]).map(|(a, b)| a - b).collect();
    let (e, a, d) = (rms(&expected[..n]), rms(&actual[..n]), rms(&diff));
    eprintln!("rms built-in {e}, Lime VST3 {a}, difference {d}");
    assert!(e > 0.01, "the built-in render is silent: {e}");
    assert!(d < 1e-4 * e, "Lime VST3 render differs: rms {a} vs {e}, difference {d}");
    assert!(
        expected[..n]
            .iter()
            .map(|v| v.to_bits())
            .eq(actual[..n].iter().map(|v| v.to_bits())),
        "the comparison must remain bit-identical"
    );
    assert!(
        expected[n..].iter().chain(&actual[n..]).all(|v| *v == 0.0),
        "render lengths may only differ by trailing silence"
    );
}

#[test]
fn effect_automation_follows_the_converted_lime_plugin() {
    if !bundles_or_skip(&["lime:LimeSynth.vst3", "lime:Lime Sampler.vst3", "lime:Lime EQ.vst3"]) {
        return;
    }
    let (mut project, pat) = project("missing.wav");
    // Song mode: the pattern and an automation clip of the EQ's mid gain (linear, in dB).
    let mut doc = Document::new(project.clone());
    doc.execute(Command::AddClip {
        pattern: pat,
        track: 0,
        start: 0,
        length: None,
    })
    .unwrap();
    let track = project.channels[0].mixer_track;
    let effect = project.mixer.tracks[track as usize].effects[0].id;
    doc.execute(Command::AddAutomationClip {
        target: AutomationTarget::EffectParam {
            track,
            effect,
            index: 2,
        },
        track: 1,
        start: 0,
        length: 3 * PPQ,
        value: None,
        name: None,
    })
    .unwrap();
    let automation = doc.project().automations[0].id;
    let points = [(0, 0.0), (3 * PPQ, 1.0)]
        .map(|(tick, value)| AutomationPoint {
            tick,
            value,
            tension: 0.0,
            mode: Default::default(),
        })
        .to_vec();
    doc.execute(Command::SetAutomationPoints { automation, points })
        .unwrap();
    project = doc.project().clone();

    let song = |p: &Project, processors| {
        let settings = RenderSettings {
            sample_rate: SR,
            range: limeshot_engine::ExportRange::Song,
            max_tail_secs: 0.5,
            block: 256,
        };
        if lime::needs_conversion(p) {
            let snap = Arc::new(reference::compile(p, pat, &limeshot_sequencer::NoSamples));
            return reference::render_offline(snap, &settings).unwrap().0;
        }
        let snap = Arc::new(compile(p, pat, &limeshot_sequencer::NoSamples));
        render_offline_with(snap, &settings, processors).unwrap().0
    };
    let expected = song(&project, Processors::default());
    // The ramp audibly changes the sound, so following it is what the comparison checks.
    let mut still = project.clone();
    still.automations.clear();
    still
        .playlist
        .clips
        .retain(|c| !matches!(c.source, ClipSource::Automation(_)));
    let unautomated = song(&still, Processors::default());
    let change: f32 = expected.iter().zip(&unautomated).map(|(a, b)| (a - b).abs()).sum();
    assert!(change > 1.0, "the automation does not change the render: {change}");
    let mut converted = project.clone();
    lime::convert_project(&mut converted, &limeshot_ui::lime::embedded_audio);
    assert!(matches!(
        converted.automations[0].target,
        AutomationTarget::PluginParam { .. }
    ));
    let host = PluginHost::new(None);
    let (processors, _keep) = host.offline(&converted, SR).unwrap();
    let actual = song(&converted, processors);
    let rms = |a: &[f32]| (a.iter().map(|v| v * v).sum::<f32>() / a.len().max(1) as f32).sqrt();
    let n = expected.len().min(actual.len());
    let diff: Vec<f32> = expected[..n].iter().zip(&actual[..n]).map(|(a, b)| a - b).collect();
    let (e, a, d) = (rms(&expected[..n]), rms(&actual[..n]), rms(&diff));
    eprintln!("automated: rms built-in {e}, Lime VST3 {a}, difference {d}");
    // The plugin smooths parameter changes and applies them per block, so the result is close to
    // the built-in render rather than identical; without the conversion it would be far off.
    let off: Vec<f32> = expected[..n]
        .iter()
        .zip(&unautomated[..n])
        .map(|(a, b)| a - b)
        .collect();
    let o = rms(&off);
    eprintln!("difference without automation {o}");
    assert!(
        d < 0.1 * o,
        "automation not followed: difference {d}, without automation {o}"
    );
}

#[test]
fn hosted_sampler_clips_offset_chase_overlap_gate_and_fade() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("ramp.wav");
    let samples: Vec<f32> = (0..48_000).flat_map(|i| [0.1 + i as f32 / 96_000.0; 2]).collect();
    limeshot_dsp::write_wav(&wav, SR, &samples, limeshot_dsp::BitDepth::Float32).unwrap();
    for (in_curve, out_curve) in [(0.0, 0.0), (0.375, -0.625), (2.0, 2.0)] {
        let mut doc = Document::default();
        doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
        let clip = ClipId(
            doc.execute(Command::AddAudioClip {
                sample: SampleRef {
                    path: wav.to_str().unwrap().into(),
                },
                name: "Ramp".into(),
                track: 0,
                start: 240,
                length: 720,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        // Trim 120 ticks of source; clip now begins at frame 9000 and lasts 15000 frames.
        doc.execute(Command::ResizeClipStart { clip, start: 360 }).unwrap();
        doc.execute(Command::SetClipFades {
            clip,
            fade_in: 120,
            fade_out: 120,
        })
        .unwrap();
        doc.execute(Command::SetClipFadeCurve {
            clip,
            out: false,
            tension: in_curve,
        })
        .unwrap();
        doc.execute(Command::SetClipFadeCurve {
            clip,
            out: true,
            tension: out_curve,
        })
        .unwrap();
        if in_curve == 2.0 {
            doc.execute(Command::SetClipSmoothFade { clip, smooth: true }).unwrap();
        }
        let channel = doc.project().clip(clip).unwrap().audio_channel().unwrap();
        doc.execute(Command::SetChannelVolume { channel, volume: 1.0 }).unwrap();
        doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
        // Same-key clip overlaps; it must neither deduplicate nor release the first one.
        doc.execute(Command::AddAudioClip {
            sample: SampleRef {
                path: wav.to_str().unwrap().into(),
            },
            name: "Ramp".into(),
            track: 1,
            start: 480,
            length: 600,
        })
        .unwrap();
        let mut project = doc.project().clone();
        assert!(lime::convert_project(&mut project, &limeshot_ui::lime::embedded_audio).is_empty());
        let snapshot = Arc::new(compile(
            &project,
            project.patterns[0].id,
            &limeshot_sequencer::NoSamples,
        ));
        let render = |block, from| {
            let host = PluginHost::new(None);
            let (processors, _keep) = host.offline(&project, SR).unwrap();
            render_offline_with(
                snapshot.clone(),
                &RenderSettings {
                    sample_rate: SR,
                    range: limeshot_engine::ExportRange::Selection { from, to: 1200 },
                    max_tail_secs: 0.0,
                    block,
                },
                processors,
            )
            .unwrap()
            .0
        };
        let baseline = render(256, 0);
        for block in [1, 64, 333, 1024] {
            let audio = render(block, 0);
            assert_eq!(audio, baseline, "block {block}");
            for frame in [
                0, 8999, 9000, 10500, 11999, 12000, 15000, 22500, 23999, 24000, 26999, 27000, 29000,
            ] {
                let mut expected = 0.0;
                if (9000..24000).contains(&frame) {
                    let pos = (frame - 9000) as f32;
                    let fade = limeshot_dsp::voice::fade_curve((pos / 3000.0).min(1.0), in_curve)
                        * limeshot_dsp::voice::fade_curve(((24000 - frame) as f32 / 3000.0).min(1.0), out_curve);
                    expected += (0.1 + (frame - 9000 + 3000) as f32 / 96_000.0) * fade;
                }
                if (12000..27000).contains(&frame) {
                    expected += 0.1 + (frame - 12000) as f32 / 96_000.0;
                }
                assert!(
                    (audio[2 * frame] - expected).abs() < 1e-5,
                    "frame {frame}: {} vs {expected}",
                    audio[2 * frame]
                );
            }
            // Starting halfway through fade-in chases source offset and fade position.
            let chased = render(block, 420);
            for frame in [0, 10, 1499, 1500, 3000, 12000, 16500] {
                assert!(
                    (chased[2 * frame] - baseline[2 * (10500 + frame)]).abs() < 1e-5,
                    "chase block {block}, frame {frame}"
                );
            }
        }
    }
}

#[test]
fn hosted_disk_sampler_offline_seeks_wait_for_read_ahead() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("disk-ramp.wav");
    let samples: Vec<f32> = (0..240_000).flat_map(|i| [i as f32 / 960_000.0; 2]).collect();
    limeshot_dsp::write_wav(&source, SR, &samples, limeshot_dsp::BitDepth::Float32).unwrap();
    let mut plugin = LimePlugin::SAMPLER.with_values(&lime::sampler_values(&SamplerParams::default(), 60), false, None);
    assert!(lime::set_disk_source(&mut plugin, &source.to_string_lossy()));
    plugin
        .extra
        .insert("lime-sample-path".into(), serde_json::json!(source.to_string_lossy()));
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let channel = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Disk ramp".into(),
            instrument: Instrument::Plugin(plugin),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::SetChannelVolume { channel, volume: 1.0 }).unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
    doc.execute(Command::PlaceClip {
        source: ClipSource::Audio(channel),
        track: 0,
        start: 0,
        length: 4 * PPQ,
        offset: 2 * PPQ,
    })
    .unwrap();
    let project = doc.project();
    let snapshot = Arc::new(compile(project, project.patterns[0].id, &limeshot_sequencer::NoSamples));
    let mut baseline: Option<Vec<f32>> = None;
    for block in [1, 64, 333, 1024] {
        let host = PluginHost::new(None);
        let (processors, _keep) = host.offline(project, SR).unwrap();
        let (audio, _) = render_offline_with(
            snapshot.clone(),
            &RenderSettings {
                sample_rate: SR,
                range: limeshot_engine::ExportRange::Selection { from: PPQ, to: 4 * PPQ },
                max_tail_secs: 0.0,
                block,
            },
            processors,
        )
        .unwrap();
        for frame in [0, 1, 4095, 8192, 71_999] {
            assert!(
                (audio[2 * frame] - samples[2 * (72_000 + frame)]).abs() < 1e-6,
                "offline disk sample at {frame}, block {block}: actual {}, expected {}",
                audio[2 * frame],
                samples[2 * (72_000 + frame)]
            );
        }
        if let Some(baseline) = &baseline {
            assert!(
                audio.iter().zip(baseline).all(|(a, b)| a == b),
                "disk seek differs at block {block}"
            );
        } else {
            baseline = Some(audio);
        }
    }
}

#[test]
fn hosted_instruments_keep_independent_note_pan() {
    if !bundles_or_skip(&["lime:LimeSynth.vst3", "lime:Lime Sampler.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("dc.wav");
    limeshot_dsp::write_wav(&wav, SR, &vec![0.5; 96_000], limeshot_dsp::BitDepth::Float32).unwrap();
    for sampler in [false, true] {
        let mut doc = Document::default();
        let pat = doc.project().patterns[0].id;
        let instrument = if sampler {
            Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: wav.to_str().unwrap().into(),
                }),
                ..Default::default()
            })
        } else {
            Instrument::Synth(SynthParams {
                attack_ms: 0.0,
                ..Default::default()
            })
        };
        let channel = ChannelId(
            doc.execute(Command::AddChannel {
                name: "Pan".into(),
                instrument,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        for (key, start, pan) in [(60, 0, -1.0), (64, 240, 1.0)] {
            let note = NoteId(
                doc.execute(Command::AddNote {
                    pattern: pat,
                    note: NewNote {
                        channel,
                        key,
                        start,
                        length: 480,
                        velocity: 1.0,
                    },
                })
                .unwrap()
                .0
                .unwrap(),
            );
            doc.execute(Command::SetNoteValues {
                pattern: pat,
                prop: NoteProp::Pan,
                values: vec![(note, pan)],
            })
            .unwrap();
        }
        let mut project = doc.project().clone();
        assert!(lime::convert_project(&mut project, &limeshot_ui::lime::embedded_audio).is_empty());
        for block in [1, 64, 333, 1024] {
            let host = PluginHost::new(None);
            let (processors, _keep) = host.offline(&project, SR).unwrap();
            let snapshot = Arc::new(compile(&project, pat, &limeshot_sequencer::NoSamples));
            let audio = render_offline_with(
                snapshot,
                &RenderSettings {
                    sample_rate: SR,
                    range: limeshot_engine::ExportRange::Pattern,
                    max_tail_secs: 0.0,
                    block,
                },
                processors,
            )
            .unwrap()
            .0;
            let energy = |from: usize, to: usize, side: usize| {
                audio[2 * from + side..2 * to]
                    .iter()
                    .step_by(2)
                    .map(|v| v * v)
                    .sum::<f32>()
            };
            assert!(energy(100, 4000, 0) > 0.1);
            assert_eq!(energy(100, 4000, 1), 0.0);
            assert!(
                energy(6000, 9000, 0) > 0.1 && energy(6000, 9000, 1) > 0.1,
                "independent overlapping voices"
            );
        }
    }
}

#[test]
fn hosted_track_eq_preserves_every_legacy_band_parameter() {
    if !bundles_or_skip(&["lime:Lime EQ.vst3"]) {
        return;
    }
    use limeshot_dsp::effects::Effect;
    for bands in [
        [
            EqBand {
                freq: 20.0,
                gain: 18.0,
                width: 0.0,
            },
            EqBand {
                freq: 19000.0,
                gain: -18.0,
                width: 1.0,
            },
            EqBand {
                freq: 50.0,
                gain: 12.0,
                width: 0.9,
            },
        ],
        [
            EqBand {
                freq: 20000.0,
                gain: -6.0,
                width: 1.0,
            },
            EqBand {
                freq: 20.0,
                gain: 18.0,
                width: 0.0,
            },
            EqBand {
                freq: 20000.0,
                gain: -18.0,
                width: 0.0,
            },
        ],
        EqBand::DEFAULTS,
    ] {
        let mut values: Vec<_> = lime::EQ_CONTROLS.iter().map(|s| s.default).collect();
        values[7] = 1.0;
        for (i, b) in bands.iter().enumerate() {
            values[8 + 3 * i..11 + 3 * i].copy_from_slice(&[b.freq, b.gain, b.width]);
        }
        let reference = lime::LimePlugin(3).with_values(&values, false, None);
        let module = limeshot_plugins::load_module(&reference.path).unwrap();
        for rate in [44100, 48000, 96000] {
            for block in [1, 64, 333, 1024] {
                let (mut inst, mut processor) =
                    limeshot_plugins::Vst3Instance::new(&module, &reference.class_id, f64::from(rate), 1024).unwrap();
                inst.set_state(reference.state.bytes()).unwrap();
                let mut dsp = limeshot_dsp::effects::Eq3::new(rate as f32);
                dsp.set_bands(&bands.map(|b| (b.freq, b.gain, b.width)));
                let signal: Vec<f32> = (0..4096)
                    .flat_map(|i| {
                        let x = (i as f32 * 0.173).sin() * 0.1;
                        [x, -x]
                    })
                    .collect();
                let mut expected = signal.clone();
                dsp.process(&mut expected);
                let mut actual = signal;
                for chunk in actual.chunks_mut(2 * block) {
                    processor.process(chunk, &[], &limeshot_dsp::Transport::default());
                }
                assert_eq!(actual, expected, "rate {rate}, block {block}");
            }
        }
    }
}

#[test]
fn hosted_comp_uses_the_mixer_key_and_reports_reduction_and_voices() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3", "lime:Lime Comp.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let key = dir.path().join("key.wav");
    let signal = dir.path().join("signal.wav");
    limeshot_dsp::write_wav(&key, SR, &vec![1.0; 96000], limeshot_dsp::BitDepth::Float32).unwrap();
    limeshot_dsp::write_wav(&signal, SR, &vec![0.01; 96000], limeshot_dsp::BitDepth::Float32).unwrap();
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    for (path, start) in [(&key, 960), (&signal, 0)] {
        let ch = ChannelId(
            doc.execute(Command::AddChannel {
                name: "DC".into(),
                instrument: Instrument::Sampler(SamplerParams {
                    sample: Some(SampleRef {
                        path: path.to_str().unwrap().into(),
                    }),
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
                start,
                length: 1920,
                velocity: 1.0,
            },
        })
        .unwrap();
    }
    doc.execute(Command::SetRoute {
        from: 1,
        to: 0,
        level: None,
    })
    .unwrap();
    doc.execute(Command::SetRoute {
        from: 1,
        to: 2,
        level: Some(0.0),
    })
    .unwrap();
    let comp = doc
        .execute(Command::AddEffect {
            track: 2,
            kind: EffectKind::Compressor,
        })
        .unwrap()
        .0
        .unwrap();
    for (index, value) in [(0, -20.0), (1, 20.0), (2, 0.1), (3, 50.0), (5, 1.0)] {
        doc.execute(Command::SetEffectParam {
            track: 2,
            effect: comp,
            index,
            value,
        })
        .unwrap();
    }
    let mut project = doc.project().clone();
    assert!(lime::convert_project(&mut project, &limeshot_ui::lime::embedded_audio).is_empty());
    for block in [1, 64, 333, 1024] {
        let host = PluginHost::new(None);
        let (processors, _keep) = host.offline(&project, SR).unwrap();
        let snapshot = Arc::new(compile(&project, pat, &limeshot_sequencer::NoSamples));
        let mut renderer = limeshot_engine::Renderer::new(SR, snapshot);
        for (id, p) in processors.instruments {
            renderer.adopt_instrument(id, p);
        }
        for (id, p) in processors.effects {
            renderer.adopt_effect(id, p);
        }
        renderer.rearrange();
        renderer.play();
        let mut audio = vec![0.0; 2 * 40000];
        let mut reduction = 0.0_f32;
        let mut voices = 0;
        for chunk in audio.chunks_mut(2 * block) {
            renderer.render(chunk);
            reduction = reduction.max(renderer.gain_reductions()[2]);
            voices = voices.max(renderer.active_voices());
        }
        let mean = |from: usize, to: usize| {
            audio[2 * from..2 * to].iter().map(|v| v.abs()).sum::<f32>() / (2 * (to - from)) as f32
        };
        assert!(mean(30000, 39000) < mean(1000, 20000) * 0.2, "block {block}");
        assert!(reduction > 10.0, "measured reduction {reduction}, block {block}");
        assert_eq!(voices, 2, "real hosted voices, block {block}");
        renderer.silence();
        assert_eq!(renderer.active_voices(), 0);
    }
}

#[test]
fn converted_delay_uses_project_and_automated_tempo_after_enabling() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3", "lime:Lime Delay.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("impulse.wav");
    limeshot_dsp::write_wav(&wav, SR, &[1.0, 1.0, 0.0, 0.0], limeshot_dsp::BitDepth::Float32).unwrap();
    for (bpm, automated, expected_frame) in [(90.0, false, 32000), (180.0, false, 16000), (90.0, true, 16000)] {
        let mut doc = Document::default();
        doc.execute(Command::SetTempo(Tempo::from_bpm(bpm))).unwrap();
        let pat = doc.project().patterns[0].id;
        let channel = ChannelId(
            doc.execute(Command::AddChannel {
                name: "Impulse".into(),
                instrument: Instrument::Sampler(SamplerParams {
                    sample: Some(SampleRef {
                        path: wav.to_str().unwrap().into(),
                    }),
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
                channel,
                key: 60,
                start: 0,
                length: 960,
                velocity: 1.0,
            },
        })
        .unwrap();
        doc.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 0,
            length: Some(3840),
        })
        .unwrap();
        let effect = doc
            .execute(Command::AddEffect {
                track: 1,
                kind: EffectKind::Delay,
            })
            .unwrap()
            .0
            .unwrap();
        for (index, value) in [(1, 0.0), (2, 0.0), (4, 1.0), (5, 5.0)] {
            doc.execute(Command::SetEffectParam {
                track: 1,
                effect,
                index,
                value,
            })
            .unwrap();
        }
        doc.execute(Command::SetEffectEnabled {
            track: 1,
            effect,
            enabled: false,
        })
        .unwrap();
        if automated {
            doc.execute(Command::AddAutomationClip {
                target: AutomationTarget::Tempo,
                track: 3,
                start: 0,
                length: 3840,
                value: None,
                name: None,
            })
            .unwrap();
            let automation = doc.project().automations.last().unwrap().id;
            doc.execute(Command::SetAutomationPoints {
                automation,
                points: vec![AutomationPoint {
                    tick: 0,
                    value: 1.0,
                    tension: 0.0,
                    mode: PointMode::Hold,
                }],
            })
            .unwrap();
        }
        let mut project = doc.project().clone();
        assert!(lime::convert_project(&mut project, &limeshot_ui::lime::embedded_audio).is_empty());
        let mut doc = Document::new(project);
        doc.execute(Command::SetEffectEnabled {
            track: 1,
            effect,
            enabled: true,
        })
        .unwrap();
        for block in [1, 64, 333, 1024] {
            let host = PluginHost::new(None);
            let (processors, _keep) = host.offline(doc.project(), SR).unwrap();
            let snapshot = Arc::new(compile(doc.project(), pat, &limeshot_sequencer::NoSamples));
            let (audio, _) = render_offline_with(
                snapshot,
                &RenderSettings {
                    sample_rate: SR,
                    range: limeshot_engine::ExportRange::Song,
                    max_tail_secs: 0.0,
                    block,
                },
                processors,
            )
            .unwrap();
            let first = audio.as_chunks::<2>().0.iter().position(|f| f[0].abs() > 1e-6).unwrap();
            assert!(
                first.abs_diff(expected_frame) <= 1,
                "tempo {bpm}, automated {automated}, block {block}: echo at {first}, expected {expected_frame}"
            );
        }
    }
}

#[test]
fn missing_bundle_gate_skips_locally_and_fails_on_ci() {
    let result = std::panic::catch_unwind(|| bundles_or_skip(&["lime:DefinitelyMissing.vst3"]));
    if std::env::var_os("CI").is_some() {
        assert!(result.is_err());
    } else {
        assert!(!result.unwrap());
    }
}

/// Closing the last instance must not unload callbacks registered by the plugin framework.
#[test]
fn loaded_modules_survive_last_client_and_concurrent_reloads() {
    if !bundles_or_skip(&["lime:LimeSynth.vst3"]) {
        return;
    }
    let path = limeshot_plugins::lime::bundle_path("lime:LimeSynth.vst3").unwrap();
    let module = limeshot_plugins::Vst3Module::load(&path).unwrap();
    let resident = Arc::downgrade(&module);
    drop(module);
    let resident = resident.upgrade().expect("module remains resident without clients");
    assert!(!resident.classes().is_empty());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let path = &path;
            let resident = &resident;
            scope.spawn(move || {
                for _ in 0..32 {
                    let loaded = limeshot_plugins::Vst3Module::load(path).unwrap();
                    assert!(Arc::ptr_eq(resident, &loaded));
                }
            });
        }
    });
}

#[test]
fn hosted_sampler_synchronizes_host_and_native_state_across_presets_and_undo() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.wav");
    let second = dir.path().join("second.wav");
    limeshot_dsp::write_wav(&first, SR, &vec![0.4; 2 * SR as usize], limeshot_dsp::BitDepth::Float32).unwrap();
    limeshot_dsp::write_wav(&second, SR, &vec![0.6; SR as usize], limeshot_dsp::BitDepth::Float32).unwrap();
    let mut app = limeshot_ui::LimeShotApp::new_silent(limeshot_ui::Settings::default(), None);
    app.samples.load_blocking(&first.to_string_lossy()).unwrap();
    let channel = limeshot_ui::browser::new_sampler_channel(&mut app, &first).unwrap();
    let target = PluginTarget::Channel(channel);
    app.sync_engine();
    assert!(app.plugins.is_loaded(target));
    let plugin = app.project().plugin(target).unwrap();
    assert_eq!(
        lime::sample_source(plugin).as_deref(),
        Some(first.to_string_lossy().as_ref()),
        "host-loaded embedded sampler also populates sample-v1.source"
    );

    // Applying a Sampler preset preserves the embedded sample and root key even after the source file is removed.
    std::fs::remove_file(&first).unwrap();
    app.exec(Command::SetChannelRootKey { channel, key: 64 });
    let before_preset = app.project().plugin(target).unwrap().state.clone();
    limeshot_ui::presets_ui::apply(
        &mut app,
        limeshot_ui::presets_ui::Target::Channel(channel),
        limeshot_project::preset::Content::Sampler(SamplerParams {
            attack_ms: 175.0,
            cutoff_hz: 1200.0,
            ..Default::default()
        }),
    );
    assert_eq!(app.project().channel(channel).unwrap().root_key, 64);
    let after_preset = lime::sampler_params(&app.project().channel(channel).unwrap().instrument).unwrap();
    assert!((after_preset.attack_ms - 175.0).abs() < 0.01);
    assert!((after_preset.cutoff_hz - 1200.0).abs() < 1.0);
    assert!(
        app.sampler_source(channel).is_some(),
        "embedded sample survives preset application without re-reading the missing file"
    );
    assert!(app.doc.undo());
    assert_eq!(app.project().plugin(target).unwrap().state, before_preset);
    assert!(app.doc.redo());

    // Apply a native state replacement with a different root key and sample source, and verify
    // that the running instance, host root key, sampler_source, and undo/redo all agree.
    let mut native_ref = LimePlugin::SAMPLER.with_values(
        &lime::sampler_values(
            &SamplerParams {
                end: 0.5,
                ..Default::default()
            },
            72,
        ),
        false,
        Some(&lime::EmbeddedAudio {
            name: "second.wav".into(),
            sample_rate: SR,
            stereo: vec![0.6; SR as usize],
        }),
    );
    assert!(lime::set_sample_source(&mut native_ref, &second.to_string_lossy()));
    app.exec(Command::SetPluginState {
        plugin: target,
        state: native_ref.state.clone(),
    });
    app.sync_engine();
    assert_eq!(app.project().channel(channel).unwrap().root_key, 72);
    assert_eq!(
        lime::sampler_params(&app.project().channel(channel).unwrap().instrument)
            .unwrap()
            .sample
            .unwrap()
            .path,
        second.to_string_lossy(),
        "native source replacement updates the host's sample path metadata"
    );
    let cached = app.sampler_source(channel).expect("embedded sample cached");
    assert!(matches!(
        app.samples.status_of_path(&cached),
        Some(limeshot_audio::assets::SampleStatus::Ready(s)) if s.frames() == SR as usize / 2
    ));
    let revision_after_replace = app.doc.revision();
    app.plugins.store_states(&mut app.doc);
    assert_eq!(
        app.doc.revision(),
        revision_after_replace,
        "saving without a new native edit does not add a redundant undo step"
    );
    assert!(app.doc.undo());
    app.sync_engine();
    assert_eq!(app.project().channel(channel).unwrap().root_key, 64);
}

/// Compare prepared overlap playback, reopen and mid-clip chase across render block sizes.
#[test]
fn prepared_clips_crossfade_reopen_and_chase_at_every_block_size() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3"]) {
        return;
    }
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.))).unwrap();
    let sampler = LimePlugin::SAMPLER.with_values(
        &lime::sampler_values(
            &SamplerParams {
                start: 0.1,
                end: 0.9,
                reverse: true,
                ping_pong: true,
                looping: true,
                loop_start: 0.2,
                loop_end: 0.8,
                release_ms: 0.,
                ..Default::default()
            },
            60,
        ),
        false,
        Some(&EmbeddedAudio {
            name: "constant stereo".into(),
            sample_rate: 48000,
            stereo: [0.2, -0.1].repeat(48000),
        }),
    );
    let original = sampler.state.clone();
    let prepared = limeshot_sequencer::preparation::prepare_sample(
        &lime::sample_state(&sampler).unwrap(),
        2.,
        0.,
        true,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    let state = lime::with_sample_state(&sampler, &prepared).unwrap();
    let channel = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Original".into(),
            instrument: Instrument::Plugin(sampler),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::SetChannelVolume { channel, volume: 1. }).unwrap();
    let route = doc.project().channel(channel).unwrap().mixer_track;
    doc.execute(Command::SetMixerVolume {
        track: route,
        volume: 1.,
    })
    .unwrap();
    let mut clips = Vec::new();
    for start in [0, 1920] {
        let clip = ClipId(
            doc.execute(Command::PlaceClip {
                source: ClipSource::Audio(channel),
                track: 0,
                start,
                length: 2880,
                offset: 120,
            })
            .unwrap()
            .0
            .unwrap(),
        );
        doc.execute(Command::PrepareAudioClip {
            clip,
            expected: original.clone(),
            state: state.clone(),
            length: None,
        })
        .unwrap();
        clips.push(clip);
    }
    let Instrument::Plugin(unchanged) = &doc.project().channel(channel).unwrap().instrument else {
        panic!()
    };
    assert_eq!(unchanged.state, original);
    doc.execute(Command::AutoCrossfadeClips(clips.clone())).unwrap();
    assert_eq!(doc.project().clip(clips[0]).unwrap().fade_out, 960);
    assert_eq!(doc.project().clip(clips[1]).unwrap().fade_in, 960);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("crossfade.lime");
    limeshot_project::save(doc.project(), &path).unwrap();
    let project = limeshot_project::load(&path).unwrap();
    assert_eq!(&project, doc.project());
    assert!(doc.undo());
    assert_eq!(doc.project().clip(clips[0]).unwrap().fade_out, 0);
    assert!(doc.redo());
    assert_eq!(&project, doc.project());
    let snapshot = Arc::new(compile(
        &project,
        project.patterns[0].id,
        &limeshot_sequencer::NoSamples,
    ));
    let render = |block, from| {
        let host = PluginHost::new(None);
        let (processors, _keep) = host.offline(&project, SR).unwrap();
        render_offline_with(
            snapshot.clone(),
            &RenderSettings {
                sample_rate: SR,
                range: limeshot_engine::ExportRange::Selection { from, to: 4800 },
                max_tail_secs: 0.,
                block,
            },
            processors,
        )
        .unwrap()
        .0
    };
    let baseline = render(64, 0);
    for block in [1, 64, 333, 1024] {
        assert_eq!(render(block, 0), baseline);
        for tick in [100, 1920, 2000, 2400, 2879, 3000, 4700] {
            let sample = tick * 25;
            assert!(
                (baseline[sample * 2] - 0.2).abs() < 1e-5,
                "tick {tick}: {}",
                baseline[sample * 2]
            );
            assert!((baseline[sample * 2 + 1] + 0.1).abs() < 1e-5);
        }
        let chased = render(block, 2400);
        for i in [0, 1, 100, 1000, 11999, 20000] {
            assert!((chased[i] - baseline[2400 * 25 * 2 + i]).abs() < 1e-5);
        }
    }
}

#[test]
fn hosted_ping_pong_and_filter_modes_match_reference_across_rates_and_blocks() {
    if !bundles_or_skip(&["lime:Lime Sampler.vst3"]) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("loop.wav");
    let data: Vec<f32> = (0..64)
        .flat_map(|i| [(i as f32 * 0.2).sin() * 0.4, (i as f32 * 0.2).cos() * 0.3])
        .collect();
    limeshot_dsp::write_wav(&path, 48000, &data, limeshot_dsp::BitDepth::Float32).unwrap();
    let mut d = Document::default();
    let pat = d.project().patterns[0].id;
    d.execute(Command::SetPatternLength {
        pattern: pat,
        steps: Some(1),
    })
    .unwrap();
    let channel = ChannelId(
        d.execute(Command::AddChannel {
            name: "Loop".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: path.to_string_lossy().into_owned(),
                }),
                looping: true,
                loop_start: 0.25,
                loop_end: 0.75,
                ping_pong: true,
                one_shot: false,
                release_ms: 0.0,
                cutoff_hz: 2000.0,
                ..Default::default()
            }),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    d.execute(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel,
            key: 69,
            start: 0,
            length: STEP_TICKS,
            velocity: 0.8,
        },
    })
    .unwrap();
    let sample = Arc::new(limeshot_dsp::load_audio(&path).unwrap());
    let host = PluginHost::new(None);
    for mode in FilterMode::ALL {
        let Instrument::Sampler(mut settings) = d.project().channel(channel).unwrap().instrument.clone() else {
            panic!("sampler")
        };
        settings.filter_mode = mode;
        settings.reverse = mode == FilterMode::BandPass;
        d.execute(Command::SetInstrument {
            channel,
            instrument: Instrument::Sampler(settings),
        })
        .unwrap();
        let mut converted = d.project().clone();
        assert!(lime::convert_project(&mut converted, &limeshot_ui::lime::embedded_audio).is_empty());
        for rate in [44100, 48000, 96000] {
            for block in [1, 64, 333, 1024] {
                let settings = RenderSettings {
                    sample_rate: rate,
                    block,
                    range: limeshot_engine::ExportRange::Pattern,
                    max_tail_secs: 0.0,
                };
                let expected = reference::render_offline(
                    Arc::new(reference::compile(d.project(), pat, &|_: &str| Some(sample.clone()))),
                    &settings,
                )
                .unwrap()
                .0;
                let (processors, _keep) = host.offline(&converted, rate).unwrap();
                let actual = render_offline_with(
                    Arc::new(compile(&converted, pat, &limeshot_sequencer::NoSamples)),
                    &settings,
                    processors,
                )
                .unwrap()
                .0;
                assert!(expected.iter().any(|x| x.abs() > 1e-5));
                assert_eq!(expected.len(), actual.len());
                assert!(
                    expected.iter().zip(&actual).all(|(a, b)| (a - b).abs() < 1e-6),
                    "{mode:?}, {rate}, {block}"
                );
            }
        }
    }
}
