//! Render the LimeShot window offscreen to a PNG (no window, no screen-recording permission).
//!
//! `cargo run -p limeshot-app --example ui_shot -- out.png [--empty] [--piano] [--mixer] [--fx] [--settings] [--melody] [--detach] [--audio] [--automation] [--synth] [--markers] [--tools] [--eq] [--sidechain] [--graph] [--zip] [--auto-zip] [--scrub] [--plugin-window] [--missing-sample] [--menu=Tools | --menu=⏵/Tools] [--tool=strum] [--scale] [--note-pan] [--record] [--count-in] [--prefs=Audio]`

use egui_kittest::Harness;
use limeshot_ui::workspace::Win;
use limeshot_ui::{LimeShotApp, Settings};

/// Render the requested UI state offscreen to the first positional PNG path (default `ui.png`).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "ui.png".into());
    let samples = limeshot_app::bundled_samples_dir();
    let mut app = LimeShotApp::new_silent(Settings::default(), samples.clone());
    if !args.iter().any(|a| a == "--empty")
        && let Some(dir) = &samples
        && let Ok(doc) = limeshot_app::demo::build(dir)
    {
        app.doc.replace(doc.project().clone());
        if let Some(c) = app.project().channels.get(3).map(|c| c.id) {
            app.select_channel(c);
        }
    }
    if args.iter().any(|a| a == "--rev1") {
        app.ui.workspace.close(Win::ChannelRack);
        app.ui.playlist.clip_focus = limeshot_ui::playlist::ClipFocus::Sample;
        app.ui.playlist.selected_tracks = vec![1, 2, 3];
        app.exec(limeshot_core::Command::SetPlaylistTracksGrouped {
            tracks: vec![1, 2, 3],
            grouped: true,
        });
        app.exec(limeshot_core::Command::SetPlaylistTrackHeight {
            track: 1,
            height: Some(92.0),
            collapsed: false,
        });
    }
    if args.iter().any(|a| a == "--auto-scroll") {
        app.set_auto_scroll(true);
    }
    if args.iter().any(|a| a == "--auto-zip") {
        app.settings.auto_zip_empty_channels = true;
        if let Some(id) = app.project().patterns.get(1).map(|p| p.id) {
            app.set_current_pattern(id);
        }
    }
    if args.iter().any(|a| a == "--zip") {
        app.ui
            .rack
            .zipped
            .extend(app.project().channels.iter().take(2).map(|c| c.id).collect::<Vec<_>>());
    }
    if args.iter().any(|a| a == "--markers") {
        // Two time markers and a time selection over bars 3–4, Channel Rack closed.
        let bar = app.project().time_signature.ticks_per_bar();
        for (tick, name) in [(0, "Intro"), (4 * bar, "Drop")] {
            app.exec(limeshot_core::Command::AddMarker {
                tick,
                name: name.into(),
            });
        }
        app.doc.mark_saved();
        app.ui.playlist.time_sel = Some((2 * bar, 4 * bar));
        app.ui.workspace.close(Win::ChannelRack);
    }
    if args.iter().any(|a| a == "--piano") {
        app.show_window(Win::PianoRoll);
    }
    if args.iter().any(|a| a == "--mixer") {
        app.show_window(Win::Mixer);
    }
    if args.iter().any(|a| a == "--fx") {
        // Show routing too: the lead also feeds insert 6 as a parallel bus.
        app.exec(limeshot_core::Command::SetRoute {
            from: 4,
            to: 6,
            level: Some(0.6),
        });
        app.doc.mark_saved();
        app.ui.mixer.selected = 4;
        app.show_window(Win::Mixer);
    }
    if args.iter().any(|a| a == "--eq") {
        // Insert 1 (the drums) with a low boost and a mid cut, its panel scrolled to the EQ.
        for (band, freq, gain, width) in [(0u8, 120.0, 6.0, 0.27), (1, 2200.0, -7.5, 0.4)] {
            app.exec(limeshot_core::Command::SetMixerEq {
                track: 1,
                band,
                value: limeshot_core::EqBand { freq, gain, width },
            });
        }
        app.doc.mark_saved();
        app.ui.mixer.selected = 1;
        app.show_window(Win::Mixer);
        app.ui.workspace.state_mut(Win::Mixer).maximized = true;
    }
    if args.iter().any(|a| a == "--sidechain") {
        // The kick (insert 1) sidechains a compressor on the bass (insert 5).
        app.exec(limeshot_core::Command::SetRoute {
            from: 1,
            to: 5,
            level: Some(0.0),
        });
        if let Some(fx) = app.exec(limeshot_core::Command::AddEffect {
            track: 5,
            kind: limeshot_core::EffectKind::Compressor,
        }) {
            app.exec(limeshot_core::Command::SetPluginSidechain {
                plugin: limeshot_core::PluginTarget::Effect { track: 5, effect: fx },
                source: 1,
            });
        }
        app.doc.mark_saved();
        app.ui.mixer.selected = 5;
        app.show_window(Win::Mixer);
        app.ui.workspace.state_mut(Win::Mixer).maximized = true;
    }
    if args.iter().any(|a| a == "--graph")
        && let Some(hat) = app.project().channels.get(2).map(|c| c.id)
    {
        // The graph editor on the hats, with an accent pattern drawn into the velocities.
        let pid = app.current_pattern();
        let values: Vec<(limeshot_core::NoteId, f32)> = app
            .project()
            .pattern(pid)
            .map(|p| {
                p.notes_for(hat)
                    .enumerate()
                    .map(|(i, n)| (n.id, if i % 2 == 0 { 0.9 } else { 0.45 }))
                    .collect()
            })
            .unwrap_or_default();
        app.exec(limeshot_core::Command::SetNoteValues {
            pattern: pid,
            prop: limeshot_core::NoteProp::Velocity,
            values,
        });
        app.doc.mark_saved();
        app.select_channel(hat);
        app.ui.rack.graph = Some(limeshot_core::NoteProp::Velocity);
    }
    if args.iter().any(|a| a == "--settings")
        && let Some(c) = app.project().channels.first().map(|c| c.id)
    {
        app.select_channel(c);
        app.show_window(Win::ChannelSettings);
    }
    if args.iter().any(|a| a == "--synth")
        && let Some(c) = app.project().channels.get(3).map(|c| c.id)
    {
        app.select_channel(c);
        app.show_window(Win::ChannelSettings);
        app.ui.workspace.state_mut(Win::ChannelSettings).maximized = true;
    }
    if args.iter().any(|a| a == "--plugin") {
        // LimeShot's built-in test plugins: a plugin channel and a plugin insert on its track.
        app.plugins.add_builtin_test_plugins();
        let entries = app.plugins.db.entries.clone();
        if let Some(tone) = entries.iter().find(|e| e.instrument) {
            limeshot_ui::plugins::add_channel(&mut app, tone);
        }
        if let Some(gain) = entries.iter().find(|e| !e.instrument) {
            app.exec(limeshot_core::Command::AddPluginEffect {
                track: 1,
                plugin: gain.plugin_ref(),
            });
        }
        app.doc.mark_saved();
        app.ui.mixer.selected = 1;
        app.show_window(Win::ChannelSettings);
        app.show_window(Win::Mixer);
    }
    if args.iter().any(|a| a == "--melody")
        && let Some(p) = app.project().patterns.get(1).map(|p| p.id)
    {
        app.set_current_pattern(p);
    }
    if args.iter().any(|a| a == "--audio")
        && let Some(dir) = &samples
    {
        // Audio clips on tracks 5 and 6, zoomed in enough to show their waveforms.
        let bar = app.project().time_signature.ticks_per_bar();
        app.samples
            .load_blocking(&dir.join("Lime Snare.wav").to_string_lossy())
            .ok();
        app.samples
            .load_blocking(&dir.join("Lime Hat Open.wav").to_string_lossy())
            .ok();
        let snare = limeshot_ui::playlist::add_audio_clip(&mut app, &dir.join("Lime Snare.wav"), 4, 2 * bar);
        limeshot_ui::playlist::add_audio_clip(&mut app, &dir.join("Lime Hat Open.wav"), 5, 2 * bar + bar / 2);
        // The snare stretched to a bar with fades, to show the fade handles.
        if let Some(clip) = snare {
            app.exec(limeshot_core::Command::ResizeClip { clip, length: bar });
            app.exec(limeshot_core::Command::SetClipFades {
                clip,
                fade_in: bar / 4,
                fade_out: bar / 2,
            });
        }
        app.ui.workspace.close(Win::ChannelRack);
        app.ui.playlist.zoom_x = 90.0;
        app.doc.mark_saved();
    }
    if args.iter().any(|a| a == "--tools") {
        // With --audio: the first audio clip sliced in two, the second slipped, the Slip tool on.
        let audio: Vec<limeshot_core::Clip> = app
            .project()
            .playlist
            .clips
            .iter()
            .filter(|c| c.audio_channel().is_some())
            .cloned()
            .collect();
        if let [a, b, ..] = audio.as_slice() {
            app.exec(limeshot_core::Command::SliceClips {
                clips: vec![a.id],
                at: a.start + a.length / 2,
            });
            app.exec(limeshot_core::Command::SlipClip {
                clip: b.id,
                offset: b.length / 4,
            });
        }
        app.ui.playlist.tool = limeshot_ui::piano_roll::Tool::Slip;
        app.doc.mark_saved();
    }
    if args.iter().any(|a| a == "--automation")
        && let Some(lead) = app.project().channels.get(3).map(|c| c.id)
        && let Some(clip) =
            limeshot_ui::automation_ui::create(&mut app, limeshot_core::AutomationTarget::ChannelPan(lead))
        && let Some(a) = app.project().clip(clip).and_then(|c| c.automation())
    {
        // A pan sweep with a curved segment, as a user would draw it, then stairs and a wave
        // (point modes).
        let bar = app.project().time_signature.ticks_per_bar();
        let pt = |tick, value, tension, mode| limeshot_core::AutomationPoint {
            tick,
            value,
            tension,
            mode,
        };
        use limeshot_core::PointMode as M;
        app.exec(limeshot_core::Command::SetAutomationPoints {
            automation: a,
            points: vec![
                pt(0, 0.5, 0.0, M::SingleCurve),
                pt(2 * bar, 0.1, 0.6, M::SingleCurve),
                pt(4 * bar, 0.9, -0.6, M::Stairs),
                pt(5 * bar, 0.5, -0.8, M::Wave),
                pt(7 * bar, 0.2, 0.0, M::SingleCurve),
            ],
        });
        app.ui.workspace.close(Win::ChannelRack);
        app.ui.playlist.zoom_x = 30.0;
        app.doc.mark_saved();
    }
    if args.iter().any(|a| a == "--detach") {
        // Without multi-viewport support the detached window renders embedded; this checks it draws.
        app.ui.workspace.state_mut(Win::ChannelRack).detached = true;
    }
    if let Some(dialog) = args.iter().find_map(|a| a.strip_prefix("--dialog=")) {
        use limeshot_ui::dialogs::Dialog;
        app.dialog = match dialog {
            "control" => Some(Dialog::ControlValue {
                target: limeshot_core::AutomationTarget::MixerVolume(1),
                text: "-12".into(),
            }),
            "export" => Some(Dialog::export(&app)),
            "mp3" => Some(Dialog::export_mp3(&app)),
            "quit" => Some(Dialog::ConfirmDiscard(limeshot_ui::app::Pending::Quit)),
            "marker" => limeshot_ui::playlist::add_marker(&mut app, 0).map(|id| Dialog::rename_marker(&app, id)),
            _ => None,
        };
    }
    if args.iter().any(|a| a == "--tapper") {
        app.ui.toolbar.tapper = Some(limeshot_ui::toolbar::Tapper::default());
    }
    if args.iter().any(|a| a == "--missing-sample")
        && let Ok(created) = app.doc.execute(limeshot_core::Command::AddChannel {
            name: "Missing sample".into(),
            instrument: limeshot_core::Instrument::Sampler(limeshot_core::SamplerParams {
                sample: Some(limeshot_core::SampleRef {
                    path: "/__limeshot_missing_source__/unavailable.wav".into(),
                }),
                ..Default::default()
            }),
        })
        && let Some(id) = created.0
    {
        app.select_channel(limeshot_core::ChannelId(id));
        app.show_window(limeshot_ui::workspace::Win::ChannelSettings);
    }
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .with_pixels_per_point(1.0)
        .wgpu()
        .build_ui_state(
            |ui, app: &mut LimeShotApp| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
            app,
        );
    for _ in 0..12 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    if args.iter().any(|a| a == "--scrub") {
        // The melody in a maximised Piano Roll with the Playback tool held over its second note:
        // the scrub line and the tool's cursor.
        let lead = harness.state().project().channels.get(3).map(|c| c.id);
        let melody = harness.state().project().patterns.get(1).map(|p| p.id);
        if let (Some(lead), Some(pat)) = (lead, melody) {
            let st = harness.state_mut();
            st.set_current_pattern(pat);
            st.select_channel(lead);
            st.show_window(Win::PianoRoll);
            st.ui.workspace.state_mut(Win::PianoRoll).maximized = true;
            st.ui.piano.tool = limeshot_ui::piano_roll::Tool::Playback;
            for _ in 0..4 {
                harness.step();
            }
            let note = harness
                .state()
                .project()
                .pattern(pat)
                .and_then(|p| p.notes_for(lead).nth(1).copied());
            if let (Some(g), Some(n)) = (harness.state().ui.piano.geometry, note) {
                let p = g.pos(n.start + n.length / 2, n.key.saturating_sub(3));
                harness.event(egui::Event::PointerMoved(p));
                harness.step();
                harness.event(egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                });
                harness.step();
                harness.step();
            }
        }
    }
    if args.iter().any(|a| a == "--plugin-window") {
        // With --plugin: the Lime Test Tone channel's plugin window (embedded without viewports).
        let st = harness.state_mut();
        let tone = st
            .project()
            .channels
            .iter()
            .find(|c| matches!(c.instrument, limeshot_core::Instrument::Plugin(_)))
            .map(|c| c.id);
        if let Some(c) = tone {
            st.plugins.open_window(limeshot_ui::plugins::Target::Channel(c), None);
            for _ in 0..4 {
                harness.step();
            }
        }
    }
    if let Some(page) = args.iter().find_map(|a| a.strip_prefix("--prefs=")) {
        // The settings window on a page ("Audio", "Info").
        use limeshot_ui::settings_window::Page;
        let pages = [
            Page::Midi,
            Page::Audio,
            Page::General,
            Page::File,
            Page::Project,
            Page::Info,
            Page::About,
        ];
        if let Some(p) = pages.into_iter().find(|p| p.label().eq_ignore_ascii_case(page)) {
            harness.state_mut().open_settings(p);
            for _ in 0..4 {
                harness.step();
            }
        }
    }
    if args.iter().any(|a| a == "--record") {
        // The record button armed.
        harness.state_mut().ui.record.armed = true;
        harness.step();
    }
    if args.iter().any(|a| a == "--count-in") {
        harness
            .state_mut()
            .exec(limeshot_core::Command::SetTempo(limeshot_core::Tempo::from_bpm(30.0)));
        harness.state_mut().ui.record.armed = true;
        harness.state_mut().ui.record.count_in = true;
        harness.state_mut().toggle_play();
        let start = std::time::Instant::now();
        while harness.state().player.status().count_in_beats() == 0 && start.elapsed().as_secs() < 3 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        harness.step();
    }
    if args.iter().any(|a| a == "--note-pan") {
        harness.state_mut().ui.piano.target_control = limeshot_core::NoteProp::Pan;
        harness.state_mut().ui.piano.keyboard_labels = limeshot_ui::piano_roll::KeyboardLabels::All;
        harness.state_mut().ui.piano.keep_labels_on_screen = true;
        harness.state_mut().ui.piano.scroll.x = 32.0;
        let pid = harness.state().current_pattern();
        let cid = harness.state().ui.selected_channel;
        let values = harness
            .state()
            .project()
            .pattern(pid)
            .map(|p| {
                p.notes
                    .iter()
                    .filter(|n| Some(n.channel) == cid)
                    .enumerate()
                    .map(|(i, n)| (n.id, [-0.8, 0.0, 0.8][i % 3]))
                    .collect()
            })
            .unwrap_or_default();
        harness.state_mut().exec(limeshot_core::Command::SetNoteValues {
            pattern: pid,
            prop: limeshot_core::NoteProp::Pan,
            values,
        });
        harness.state_mut().ui.workspace.state_mut(Win::PianoRoll).maximized = true;
    }
    if args.iter().any(|a| a == "--scale") {
        // Piano Roll scale highlighting: A minor.
        harness.state_mut().ui.piano.scale = Some((9, limeshot_core::note_tools::Scale::Minor));
        harness.state_mut().ui.piano.stamp = Some(limeshot_core::note_tools::Stamp::Chord(1));
        for _ in 0..2 {
            harness.step();
        }
    }
    if let Some(tool) = args.iter().find_map(|a| a.strip_prefix("--tool=")) {
        // A Piano Roll tool dialog (with --piano --melody), e.g. --tool=strum.
        use limeshot_ui::piano_tools::Kind;
        let kinds = [
            ("quantize", Kind::Quantize),
            ("chop", Kind::Chop),
            ("articulate", Kind::Articulate),
            ("arpeggiate", Kind::Arpeggiate),
            ("strum", Kind::Strum),
            ("flam", Kind::Flam),
            ("limit", Kind::Limit),
            ("flip", Kind::Flip),
            ("randomize", Kind::Randomize),
            ("scale", Kind::ScaleLevels),
            ("lfo", Kind::Lfo),
        ];
        if let Some((_, k)) = kinds.iter().find(|(n, _)| *n == tool) {
            limeshot_ui::piano_tools::open(harness.state_mut(), *k);
            for _ in 0..4 {
                harness.step();
            }
        }
    }
    if let Some(menu) = args.iter().find_map(|a| a.strip_prefix("--menu=")) {
        // Open a menu, to look at its items; "⏵/Tools" opens an editor's menu (with --piano, the
        // Piano Roll's) and hovers its Tools submenu.
        use egui_kittest::kittest::Queryable;
        let mut path = menu.split('/');
        if let Some(first) = path.next() {
            // Each editor has a ⏵ menu: the last one drawn is the frontmost window's.
            if let Some(node) = harness.get_all_by_label(first).last() {
                node.click();
            }
        }
        for _ in 0..3 {
            harness.step();
        }
        for sub in path {
            harness.get_by_label(&format!("{sub} ⏵")).hover();
            for _ in 0..3 {
                harness.step();
            }
        }
    }
    match harness.render() {
        Ok(img) => match img.save(&out) {
            Ok(()) => println!("wrote {out}"),
            Err(e) => eprintln!("save: {e}"),
        },
        Err(e) => eprintln!("render: {e}"),
    }
}
