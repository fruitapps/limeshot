//! CLI saves use portable Lime references, and failed plugin loading cannot write a silent export.
use limeshot_core::{Command, EffectKind, Instrument, LimePlugin, PluginRef, SynthParams, lime};

#[test]
fn scripted_and_demo_saves_contain_lime_plugins() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("commands.json");
    let saved = dir.path().join("scripted.lime");
    let commands = vec![
        Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams::default()),
        },
        Command::AddEffect {
            track: 1,
            kind: EffectKind::Delay,
        },
    ];
    std::fs::write(&script, serde_json::to_vec(&commands).unwrap()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
        .arg("run")
        .arg(&script)
        .arg("--save")
        .arg(&saved)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let project = limeshot_project::load(&saved).unwrap();
    assert!(!lime::needs_conversion(&project));
    let Instrument::Plugin(plugin) = &project.channels[0].instrument else {
        panic!("Lime reference expected")
    };
    assert_eq!(LimePlugin::of(plugin), Some(LimePlugin::SYNTH));
    assert_eq!(project.mixer.tracks[1].effects[0].kind, EffectKind::Plugin);

    let demo = dir.path().join("demo.lime");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
        .arg("demo")
        .arg(&demo)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let project = limeshot_project::load(&demo).unwrap();
    assert!(!project.channels.is_empty());
    assert!(!lime::needs_conversion(&project));
    assert!(
        project
            .channels
            .iter()
            .all(|c| matches!(&c.instrument, Instrument::Plugin(p) if LimePlugin::of(p).is_some()))
    );
}

#[test]
fn missing_plugin_fails_export_without_creating_output() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("missing.json");
    let wav = dir.path().join("missing.wav");
    let command = Command::AddChannel {
        name: "Missing".into(),
        instrument: Instrument::Plugin(PluginRef::vst3(
            "/nonexistent/Instrument.vst3",
            "00",
            "Missing instrument",
        )),
    };
    std::fs::write(&script, serde_json::to_vec(&vec![command]).unwrap()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
        .arg("run")
        .arg(&script)
        .arg("--render")
        .arg(&wav)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("Missing instrument") && error.contains("export"),
        "{error}"
    );
    assert!(!wav.exists());
}

#[test]
fn external_sample_embedding_requires_explicit_cli_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sample = outside.path().join("private.wav");
    limeshot_dsp::write_wav(&sample, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
    let mut doc = limeshot_core::Document::default();
    doc.execute(Command::AddChannel {
        name: "External".into(),
        instrument: Instrument::Sampler(limeshot_core::SamplerParams {
            sample: Some(limeshot_core::SampleRef {
                path: sample.to_string_lossy().into_owned(),
            }),
            ..Default::default()
        }),
    })
    .unwrap();
    let source = dir.path().join("legacy.lime");
    limeshot_project::save(doc.project(), &source).unwrap();
    let script = dir.path().join("noop.json");
    std::fs::write(&script, "[]").unwrap();
    for allow in [false, true] {
        let saved = dir.path().join(if allow { "allowed.lime" } else { "blocked.lime" });
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"));
        cmd.arg("run")
            .arg(&script)
            .arg("--project")
            .arg(&source)
            .arg("--save")
            .arg(&saved);
        if allow {
            cmd.arg("--allow-external-samples");
        }
        let out = cmd.output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let project = limeshot_project::load(&saved).unwrap();
        let Instrument::Plugin(plugin) = &project.channels[0].instrument else {
            panic!("not hosted")
        };
        let state: serde_json::Value = serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
        assert_eq!(state["fields"].get("sample-v1").is_some(), allow);
        assert_eq!(plugin.extra.get("lime-sample-pending").is_some(), !allow);
    }
}

#[test]
fn flp_external_samples_require_opt_in_while_local_samples_still_embed() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let local = dir.path().join("local.wav");
    let external = outside.path().join("external.wav");
    let mut flp = limeshot_project::flp::writer::Writer::new();
    for path in [&local, &external] {
        limeshot_dsp::write_wav(path, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
        flp.channel(0, "Sample", Some(path.to_str().unwrap()), 12800, 6400, 1);
    }
    let source = dir.path().join("source.flp");
    std::fs::write(&source, flp.finish(96)).unwrap();
    for allow in [false, true] {
        let saved = dir.path().join(if allow { "allowed.lime" } else { "blocked.lime" });
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"));
        command.arg("import-flp").arg(&source).arg(&saved);
        if allow {
            command.arg("--allow-external-samples");
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let project = limeshot_project::load(&saved).unwrap();
        assert_eq!(project.channels.len(), 2);
        for (channel, embedded) in project.channels.iter().zip([true, allow]) {
            let Instrument::Plugin(plugin) = &channel.instrument else {
                panic!("not hosted")
            };
            let state: serde_json::Value =
                serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
            assert_eq!(state["fields"].get("sample-v1").is_some(), embedded);
            assert_eq!(plugin.extra.get("lime-sample-pending").is_some(), !embedded);
        }
    }
}
