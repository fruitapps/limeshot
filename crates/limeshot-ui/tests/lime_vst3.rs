//! The Lime VST3 plugins sound like the former built-in instruments and effects they replace
//! (decision D39): a project rendered with the built-ins and the same project converted to Lime
//! plugins and rendered through the plugin host give the same audio. Needs the bundles
//! (`python build.py` in `native-plugins/`); without them the test reports that and passes.
use std::sync::Arc;

use limeshot_core::*;
use limeshot_engine::{Processors, RenderSettings, render_offline_with};
use limeshot_sequencer::compile;
use limeshot_ui::plugins::PluginHost;

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
    let expected = render(&built_in, pat, Processors::default());

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
