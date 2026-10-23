//! The Lime VST3 plugins sound like the former built-in instruments and effects they replace
//! (decision D39): a project rendered with the built-ins and the same project converted to Lime
//! plugins and rendered through the plugin host give the same audio. Needs the bundles
//! (`python build.py` in `native-plugins/`); without them the test reports that and passes.
use std::sync::Arc;

use limeshot_core::*;
use limeshot_engine::{Processors, RenderSettings, render_offline_with};
use limeshot_sequencer::compile;
use limeshot_ui::plugins::PluginHost;

#[path = "../../limeshot-engine/tests/support/mod.rs"]
mod reference;

const SR: u32 = 48_000;

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
                sample: Some(SampleRef { path: wav.into() }),
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
    if limeshot_plugins::lime::bundle_path("lime:LimeSynth.vst3").is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must build the Lime VST3 bundles first"
        );
        eprintln!("Lime VST3 bundles not built; skipping (run `python build.py` in native-plugins)");
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
    if limeshot_plugins::lime::bundle_path("lime:Lime EQ.vst3").is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must build the Lime VST3 bundles first"
        );
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
    assert!(
        limeshot_plugins::lime::bundle_path("lime:Lime Sampler.vst3").is_some(),
        "build native bundles before running hosted acceptance tests"
    );
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("ramp.wav");
    let samples: Vec<f32> = (0..48_000).flat_map(|i| [0.1 + i as f32 / 96_000.0; 2]).collect();
    limeshot_dsp::write_wav(&wav, SR, &samples, limeshot_dsp::BitDepth::Float32).unwrap();
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
                let fade = (pos / 3000.0).min(1.0) * ((24000 - frame) as f32 / 3000.0).min(1.0);
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

#[test]
fn hosted_instruments_keep_independent_note_pan() {
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
