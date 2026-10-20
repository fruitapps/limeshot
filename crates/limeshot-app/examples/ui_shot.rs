//! Render the LimeShot window offscreen to a PNG (no window, no screen-recording permission).
//!
//! `cargo run -p limeshot-app --example ui_shot -- out.png [--empty] [--piano] [--mixer] [--settings] [--melody]`

use egui_kittest::Harness;
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
        app.ui.selected_channel = app.project().channels.get(3).map(|c| c.id);
    }
    if args.iter().any(|a| a == "--piano") {
        app.show_tab(limeshot_ui::app::DockTab::PianoRoll);
    }
    if args.iter().any(|a| a == "--mixer") {
        app.show_tab(limeshot_ui::app::DockTab::Mixer);
    }
    if args.iter().any(|a| a == "--settings")
        && let Some(c) = app.project().channels.first().map(|c| c.id)
    {
        app.select_channel(c);
    }
    if args.iter().any(|a| a == "--melody")
        && let Some(p) = app.project().patterns.get(1).map(|p| p.id)
    {
        app.set_current_pattern(p);
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
