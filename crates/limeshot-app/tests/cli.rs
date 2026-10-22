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
