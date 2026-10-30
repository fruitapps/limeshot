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

#[test]
fn cli_mp3_export_uses_hosted_mix_and_rejects_invalid_settings() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("demo.lime");
    let mp3 = dir.path().join("demo.MP3");
    let run = |args: &[&std::ffi::OsStr]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
            .args(args)
            .output()
            .unwrap()
    };
    let output = run(&[
        "demo".as_ref(),
        project.as_os_str(),
        "--render".as_ref(),
        mp3.as_os_str(),
        "--rate".as_ref(),
        "48000".as_ref(),
        "--bitrate".as_ref(),
        "192".as_ref(),
    ]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let decoded = limeshot_dsp::load_audio(&mp3).unwrap();
    assert_eq!(decoded.sample_rate, 48000);
    assert!(decoded.frames() > 48000);
    assert!(decoded.peak() > 0.01);
    assert!(decoded.data.iter().all(|v| v.is_finite()));
    let original = std::fs::read(&mp3).unwrap();
    for options in [["--rate", "96000"], ["--bitrate", "123"]] {
        let output = run(&[
            "render".as_ref(),
            project.as_os_str(),
            mp3.as_os_str(),
            options[0].as_ref(),
            options[1].as_ref(),
        ]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("MP3"));
        assert_eq!(std::fs::read(&mp3).unwrap(), original);
    }
    let invalid = dir.path().join("out.flac");
    let output = run(&["render".as_ref(), project.as_os_str(), invalid.as_os_str()]);
    assert!(!output.status.success());
    assert!(!invalid.exists());
}

#[test]
fn cli_split_export_writes_dithered_mixer_tracks_beside_the_master() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("demo.lime");
    let wav = dir.path().join("demo.wav");
    let run = |args: &[&std::ffi::OsStr]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
            .args(args)
            .output()
            .unwrap()
    };
    let output = run(&["demo".as_ref(), project.as_os_str()]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let output = run(&[
        "render".as_ref(),
        project.as_os_str(),
        wav.as_os_str(),
        "--bits".as_ref(),
        "16".as_ref(),
        "--dither".as_ref(),
        "--split".as_ref(),
    ]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let master = limeshot_dsp::load_wav(&wav).unwrap();
    assert!(master.peak() > 0.01);
    let mut stems: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("demo - "))
        .collect();
    stems.sort();
    // The demo's five channels play through their own inserts.
    let project_data = limeshot_project::load(&project).unwrap();
    let expected: Vec<_> = (1..=5)
        .map(|t| limeshot_engine::stem_path(&wav, t, &project_data.mixer.tracks[t].name))
        .collect();
    assert_eq!(stems, expected);
    for stem in &stems {
        let audio = limeshot_dsp::load_wav(stem).unwrap();
        assert_eq!(audio.frames(), master.frames(), "{}", stem.display());
        assert!(audio.peak() > 0.001, "{} has sound", stem.display());
        let spec = limeshot_dsp::hound::WavReader::open(stem).unwrap().spec();
        assert_eq!(spec.bits_per_sample, 16);
    }
    // MP3 cannot be split; nothing is written.
    let mp3 = dir.path().join("demo.mp3");
    let output = run(&[
        "render".as_ref(),
        project.as_os_str(),
        mp3.as_os_str(),
        "--split".as_ref(),
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("WAV"));
    assert!(!mp3.exists());
}

#[test]
fn cli_wrapped_split_export_keeps_every_file_at_the_music_length() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("demo.lime");
    let run = |args: &[&std::ffi::OsStr]| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_limeshot-cli"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    };
    run(&["demo".as_ref(), project.as_os_str()]);
    let cut = dir.path().join("cut.wav");
    let wrap = dir.path().join("wrap.wav");
    run(&[
        "render".as_ref(),
        project.as_os_str(),
        cut.as_os_str(),
        "--tail".as_ref(),
        "cut".as_ref(),
    ]);
    run(&[
        "render".as_ref(),
        project.as_os_str(),
        wrap.as_os_str(),
        "--tail".as_ref(),
        "wrap".as_ref(),
        "--split".as_ref(),
        "--bits".as_ref(),
        "24".as_ref(),
    ]);
    let music = limeshot_dsp::load_wav(&cut).unwrap().frames();
    assert_eq!(limeshot_dsp::load_wav(&wrap).unwrap().frames(), music);
    let stems: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("wrap - "))
        .collect();
    assert_eq!(stems.len(), 5);
    for stem in stems {
        assert_eq!(
            limeshot_dsp::load_wav(&stem).unwrap().frames(),
            music,
            "{}",
            stem.display()
        );
        let spec = limeshot_dsp::hound::WavReader::open(&stem).unwrap().spec();
        assert_eq!(
            spec.bits_per_sample, 24,
            "wrapped stems are written in the chosen format"
        );
    }
}
