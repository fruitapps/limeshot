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
