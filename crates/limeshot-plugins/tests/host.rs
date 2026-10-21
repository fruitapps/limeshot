//! Hosting LimeShot's built-in test plugins through the full VST3 path.

use limeshot_plugins::{NoteEvent, NoteEventKind, PluginKind, Transport, Vst3Instance, testing};

const SR: f64 = 48_000.0;

fn gain(max_block: usize) -> (Vst3Instance, limeshot_plugins::Vst3Processor) {
    let m = testing::module().unwrap();
    Vst3Instance::new(&m, &testing::gain_class(), SR, max_block).unwrap()
}

#[test]
fn the_factory_lists_its_instrument_and_effect() {
    let classes = testing::module().unwrap().classes();
    let names: Vec<_> = classes.iter().map(|c| (c.name.as_str(), c.kind)).collect();
    assert_eq!(
        names,
        [
            ("Lime Test Gain", PluginKind::Effect),
            ("Lime Test Tone", PluginKind::Instrument)
        ]
    );
    assert_eq!(classes[0].class_id, testing::gain_class());
    assert_eq!(classes[1].sub_categories, "Instrument|Synth");
    assert_eq!(classes[0].vendor, "LimeShot");
}

#[test]
fn an_unknown_class_is_an_error() {
    let m = testing::module().unwrap();
    assert!(Vst3Instance::new(&m, "00000000000000000000000000000000", SR, 64).is_err());
    assert!(Vst3Instance::new(&m, "not hex", SR, 64).is_err());
}

#[test]
fn an_effect_processes_in_place_at_any_block_size() {
    for (max_block, len) in [(64, 64), (64, 200), (256, 1), (1024, 1000)] {
        let (mut inst, mut proc) = gain(max_block);
        assert_eq!(inst.params().len(), 1);
        assert_eq!(inst.params()[0].title, "Gain");
        assert!(inst.params()[0].automatable());
        let mut buf: Vec<f32> = (0..2 * len).map(|i| if i % 2 == 0 { 0.5 } else { -0.25 }).collect();
        proc.process(&mut buf, &[], &Transport::default());
        assert!(
            buf.chunks(2).all(|f| f == [0.5, -0.25]),
            "unity gain, block {max_block}"
        );

        inst.set_param(0, 0.5);
        assert_eq!(inst.param(0), 0.5);
        assert_eq!(inst.param_text(0, 0.25), "25%");
        proc.process(&mut buf, &[], &Transport::default());
        assert!(
            buf.chunks(2).all(|f| f == [0.25, -0.125]),
            "half gain, block {max_block}"
        );
    }
}

#[test]
fn the_effect_trait_runs_the_plugin() {
    use limeshot_dsp::effects::Effect;
    let (mut inst, proc) = gain(128);
    let mut fx: Box<dyn Effect> = Box::new(proc);
    inst.set_param(0, 0.0);
    let mut buf = vec![1.0f32; 512];
    fx.process(&mut buf);
    assert!(buf.iter().all(|&s| s == 0.0));
    fx.reset();
}

#[test]
fn state_round_trips_into_a_new_instance() {
    let (mut a, _pa) = gain(64);
    a.set_param(0, 0.3);
    // The processor learns the value at its next block; the controller already has it.
    let (_, mut pa) = (0, _pa);
    let mut buf = vec![1.0f32; 2];
    pa.process(&mut buf, &[], &Transport::default());
    let blob = a.state().unwrap();
    assert!(blob.starts_with(b"LSV3"));

    let (mut b, mut pb) = gain(64);
    assert_eq!(b.param(0), 1.0);
    b.set_state(&blob).unwrap();
    assert!((b.param(0) - 0.3).abs() < 1e-12, "controller learned the state");
    let mut buf = vec![1.0f32; 8];
    pb.process(&mut buf, &[], &Transport::default());
    assert!(
        buf.iter().all(|&s| (s - 0.3).abs() < 1e-6),
        "processor learned it: {buf:?}"
    );
    assert!(b.set_state(b"garbage").is_err());
}

fn crossings(x: &[f32]) -> usize {
    x.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count()
}

#[test]
fn an_instrument_plays_notes_at_their_offsets() {
    let m = testing::module().unwrap();
    for max_block in [32, 480, 1024] {
        let (inst, mut proc) = Vst3Instance::new(&m, &testing::tone_class(), SR, max_block).unwrap();
        assert_eq!(inst.params()[0].title, "Level");
        let frames = 4800;
        let mut buf = vec![0.0f32; 2 * frames];
        let notes = [
            NoteEvent {
                offset: 100,
                kind: NoteEventKind::On {
                    key: 69,
                    velocity: 1.0,
                    id: 1,
                },
            },
            NoteEvent {
                offset: 2500,
                kind: NoteEventKind::Off { key: 69, id: 1 },
            },
        ];
        proc.process(&mut buf, &notes, &Transport::default());
        let left: Vec<f32> = buf.chunks(2).map(|f| f[0]).collect();
        assert!(left[..100].iter().all(|&s| s == 0.0), "silent before the note");
        assert!(left[2500..].iter().all(|&s| s == 0.0), "silent after note off");
        let peak = left[100..2500].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 0.5).abs() < 0.01, "level 0.5 × velocity 1, got {peak}");
        // 2400 frames of 440 Hz at 48 kHz are 22 cycles.
        let c = crossings(&left[100..2500]);
        assert!((21..=23).contains(&c), "about 22 cycles, got {c} (block {max_block})");
        assert!(buf.chunks(2).all(|f| f[0] == f[1]));
    }
}

#[test]
fn an_editor_opens_takes_edits_and_resizes_itself() {
    let (mut inst, mut proc) = gain(64);
    let mut editor = inst.create_editor().expect("Lime Test Gain has an editor");
    let (w, h) = testing::EDITOR_SIZE;
    assert_eq!(
        editor.size(),
        limeshot_plugins::EditorSize {
            width: w as u32,
            height: h as u32
        }
    );
    assert!(editor.can_resize());
    assert!(!editor.is_attached());
    editor.attach_headless().unwrap();
    assert!(editor.is_attached());

    // An edit in the editor reaches the controller, the host and, at the next block, the audio.
    assert!(editor.key_down('-'));
    assert_eq!(inst.param(0), 0.5);
    assert_eq!(inst.take_editor_edits(), [(0, 0.5)]);
    assert!(inst.take_editor_edits().is_empty());
    let mut buf = vec![1.0f32; 2 * 64];
    proc.process(&mut buf, &[], &Transport::default());
    assert!(buf.iter().all(|&s| s == 0.5));

    // The editor asks for another size; the host takes it and tells the view.
    assert_eq!(editor.take_resized(), None);
    assert!(editor.key_down('r'));
    let (w, h) = testing::EDITOR_RESIZED;
    let resized = limeshot_plugins::EditorSize {
        width: w as u32,
        height: h as u32,
    };
    assert_eq!(editor.take_resized(), Some(resized));
    assert_eq!(editor.take_resized(), None);
    assert_eq!(editor.size(), resized);

    editor.detach();
    assert!(!editor.is_attached());
    // The editor keeps the plugin alive; dropping the instance first is fine.
    drop(inst);
    editor.attach_headless().unwrap();
    drop(editor);
}

#[test]
fn a_plugin_without_an_editor_has_none() {
    let m = testing::module().unwrap();
    let (inst, _proc) = Vst3Instance::new(&m, &testing::tone_class(), SR, 64).unwrap();
    assert!(inst.create_editor().is_none());
    assert!(limeshot_plugins::NativeWindow::find("no window has this title").is_none());
}
