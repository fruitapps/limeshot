//! Render the LimeShot window offscreen to a PNG (no window, no screen-recording permission).
//!
//! `cargo run -p limeshot-app --example ui_shot -- out.png [--empty] [--piano] [--mixer] [--fx] [--settings] [--melody] [--detach] [--audio] [--automation] [--synth] [--markers] [--tools] [--eq]`

use egui_kittest::Harness;
use limeshot_ui::workspace::Win;
use limeshot_ui::{LimeShotApp, Settings};

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
        limeshot_ui::playlist::add_audio_clip(&mut app, &dir.join("Lime Snare.wav"), 4, 2 * bar);
        limeshot_ui::playlist::add_audio_clip(&mut app, &dir.join("Lime Hat Open.wav"), 5, 2 * bar + bar / 2);
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
        // A pan sweep with a curved segment, as a user would draw it.
        let bar = app.project().time_signature.ticks_per_bar();
        let pt = |tick, value, tension| limeshot_core::AutomationPoint { tick, value, tension };
        app.exec(limeshot_core::Command::SetAutomationPoints {
            automation: a,
            points: vec![
                pt(0, 0.5, 0.0),
                pt(2 * bar, 0.1, 0.6),
                pt(5 * bar, 0.9, 0.0),
                pt(7 * bar, 0.5, -0.5),
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
    match harness.render() {
        Ok(img) => match img.save(&out) {
            Ok(()) => println!("wrote {out}"),
            Err(e) => eprintln!("save: {e}"),
        },
        Err(e) => eprintln!("render: {e}"),
    }
}
