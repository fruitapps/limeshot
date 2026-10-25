//! Plugin channels and insert slots in the project model.

use limeshot_core::*;

fn plugin(name: &str) -> PluginRef {
    PluginRef::vst3(
        format!("/Plug-Ins/{name}.vst3"),
        "00112233445566778899AABBCCDDEEFF",
        name,
    )
}

#[test]
fn plugin_channels_and_effects_survive_json_and_sanitize() {
    let mut doc = Document::default();
    let ch = doc
        .execute(Command::AddChannel {
            name: "Keys".into(),
            instrument: Instrument::Plugin(plugin("Synth")),
        })
        .unwrap()
        .0
        .map(ChannelId)
        .unwrap();
    let fx = doc
        .execute(Command::AddPluginEffect {
            track: 1,
            plugin: plugin("Verb"),
        })
        .unwrap()
        .0
        .unwrap();
    assert_eq!(doc.undo_label(), Some("Add plugin"));

    // Stored state is not an edit.
    let revision = doc.revision();
    assert!(doc.store_plugin_state(PluginTarget::Channel(ch), PluginState::new(vec![1, 2, 3])));
    assert!(doc.store_plugin_state(PluginTarget::Effect { track: 1, effect: fx }, PluginState::new(vec![9])));
    assert!(!doc.store_plugin_state(
        PluginTarget::Effect {
            track: 1,
            effect: fx + 100
        },
        PluginState::default()
    ));
    assert_eq!(doc.revision(), revision);
    assert_eq!(doc.undo_label(), Some("Add plugin"));

    let json = serde_json::to_string(doc.project()).unwrap();
    let mut back: Project = serde_json::from_str(&json).unwrap();
    back.sanitize();
    let c = back.channels.iter().find(|c| c.id == ch).unwrap();
    match &c.instrument {
        Instrument::Plugin(p) => {
            assert_eq!(p.name, "Synth");
            assert_eq!(p.state.bytes(), [1, 2, 3]);
        }
        other => panic!("not a plugin: {other:?}"),
    }
    let slot = back.mixer.tracks[1].effects.iter().find(|e| e.id == fx).unwrap();
    assert_eq!(slot.kind, EffectKind::Plugin);
    assert!(slot.params.is_empty());
    assert_eq!(slot.plugin.as_ref().unwrap().state.bytes(), [9]);
}

#[test]
fn a_plugin_on_a_built_in_effect_slot_is_dropped() {
    let mut slot = EffectSlot::new(3, EffectKind::Delay);
    slot.plugin = Some(plugin("Stray"));
    slot.sanitize();
    assert!(slot.plugin.is_none());
    assert_eq!(slot.params.len(), EffectKind::Delay.params().len());
}

#[test]
fn hosted_sidechains_follow_mixer_edits_and_undo() {
    let mut doc = Document::default();
    let effect = doc
        .execute(Command::AddPluginEffect {
            track: 1,
            plugin: lime::LimePlugin(7).with_values(&[], false, None),
        })
        .unwrap()
        .0
        .unwrap();
    doc.execute(Command::SetPluginSidechain {
        plugin: PluginTarget::Effect { track: 1, effect },
        source: 3,
    })
    .unwrap();
    let source = |doc: &Document, expected: u16| {
        let p = doc
            .project()
            .mixer
            .tracks
            .iter()
            .flat_map(|t| &t.effects)
            .find(|e| e.id == effect)
            .unwrap()
            .plugin
            .as_ref()
            .unwrap();
        assert_eq!(p.extra["lime-sidechain-track"], serde_json::json!(expected));
        let key = b"key"
            .iter()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)));
        assert_eq!(
            lime::parameter_value(p, key),
            Some(if expected == 0 { 0.0 } else { 1.0 })
        );
    };
    source(&doc, 3);
    doc.execute(Command::MoveMixerTrack { track: 3, to: 2 }).unwrap();
    source(&doc, 2);
    doc.execute(Command::InsertMixerTrack(2)).unwrap();
    source(&doc, 3);
    doc.execute(Command::RemoveMixerTrack(3)).unwrap();
    source(&doc, 0);
    doc.undo();
    source(&doc, 3);
    let back: Project = serde_json::from_str(&serde_json::to_string(doc.project()).unwrap()).unwrap();
    source(&Document::new(back), 3);
}

#[test]
fn a_converted_disabled_effect_can_be_enabled() {
    let mut project = Project::default();
    let mut effect = EffectSlot::new(42, EffectKind::Delay);
    effect.enabled = false;
    project.mixer.tracks[1].effects.push(effect);
    assert!(lime::convert_project(&mut project, &|_| None).is_empty());
    let mut doc = Document::new(project);
    doc.execute(Command::SetEffectEnabled {
        track: 1,
        effect: 42,
        enabled: true,
    })
    .unwrap();
    let slot = &doc.project().mixer.tracks[1].effects[0];
    assert!(slot.enabled);
    let key = b"bypass"
        .iter()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)))
        & 0x7fff_ffff;
    assert_eq!(lime::parameter_value(slot.plugin.as_ref().unwrap(), key), Some(0.0));
    doc.undo();
    assert!(!doc.project().mixer.tracks[1].effects[0].enabled);
}

#[test]
fn effect_presets_preserve_hosted_bypass_key_and_fields() {
    let mut plugin = lime::LimePlugin(7).with_values(&[], true, None);
    let key = b"key"
        .iter()
        .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(*b)));
    plugin.state = lime::edited_param_state(&plugin, key, 1.0).unwrap();
    plugin.extra.insert("lime-sidechain-track".into(), serde_json::json!(3));
    let before: serde_json::Value = serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
    plugin.state = lime::edited_values_state(&plugin, &[-25.0, 8.0]).unwrap();
    let after: serde_json::Value = serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
    assert_eq!(after["params"]["value_1"]["f32"], -25.0);
    assert_eq!(after["params"]["bypass"], before["params"]["bypass"]);
    assert_eq!(after["params"]["key"], before["params"]["key"]);
    assert_eq!(after["fields"], before["fields"]);
    assert_eq!(plugin.extra["lime-sidechain-track"], 3);
}

#[test]
fn conversion_rejects_audio_outside_the_native_sampler_limits_with_a_warning() {
    let mut project = Project::default();
    let mut doc = Document::new(project);
    doc.execute(Command::AddChannel {
        name: "Long".into(),
        instrument: Instrument::Sampler(SamplerParams {
            sample: Some(SampleRef {
                path: "oversized.wav".into(),
            }),
            ..Default::default()
        }),
    })
    .unwrap();
    project = doc.project().clone();
    let audio = EmbeddedAudio {
        name: "oversized.wav".into(),
        sample_rate: 48000,
        stereo: vec![0.0; 2 * (lime::MAX_EMBEDDED_FRAMES + 1)],
    };
    assert!(!lime::embedded_audio_supported(&audio));
    let warnings = lime::convert_project(&mut project, &|_| Some(audio.clone()));
    assert_eq!(warnings.len(), 1, "one precise warning per rejected sample");
    assert!(warnings.iter().any(|w| w.contains("state limits")));
    let Instrument::Plugin(plugin) = &project.channels[0].instrument else {
        panic!("not converted")
    };
    assert_eq!(plugin.extra["lime-sample-path"], "oversized.wav");
    let state: serde_json::Value = serde_json::from_slice(lime::component_state(&plugin.state).unwrap()).unwrap();
    assert!(state["fields"].get("sample-v1").is_none());
    assert!(plugin.state.bytes().len() < 4096);
}

#[test]
fn insert_mix_defaults_sanitizes_persists_and_undoes() {
    let mut doc = Document::default();
    let id = doc
        .execute(Command::AddPluginEffect {
            track: 1,
            plugin: plugin("Verb"),
        })
        .unwrap()
        .0
        .unwrap();
    doc.execute(Command::SetEffectMix {
        track: 1,
        effect: id,
        mix: 0.3,
    })
    .unwrap();
    assert_eq!(doc.project().mixer.tracks[1].effects[0].mix, 0.3);
    let json = serde_json::to_string(doc.project()).unwrap();
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mixer.tracks[1].effects[0].mix, 0.3);
    doc.undo();
    assert_eq!(doc.project().mixer.tracks[1].effects[0].mix, 1.0);
    let mut slot = EffectSlot::new(1, EffectKind::Delay);
    slot.mix = f32::NAN;
    slot.sanitize();
    assert_eq!(slot.mix, 1.0);
    slot.mix = 3.0;
    slot.sanitize();
    assert_eq!(slot.mix, 1.0);
    let mut old = serde_json::to_value(&slot).unwrap();
    old.as_object_mut().unwrap().remove("mix");
    assert_eq!(serde_json::from_value::<EffectSlot>(old).unwrap().mix, 1.0);
}
