//! The LimeShot desktop application.
//!
//! `limeshot [project.lime | song.flp] [--demo] [--show=WINDOW]… [--detach=WINDOW]…`
//!
//! WINDOW is `playlist`, `rack`, `piano`, `mixer` or `settings` (window layout at start-up;
//! useful for demos and for checking detached windows). An FL Studio `.flp` is imported as a new
//! unsaved project.

use limeshot_ui::workspace::Win;
use limeshot_ui::{LimeShotApp, Settings};

struct App(LimeShotApp);

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.0.settings.save();
    }
}

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The plugin scanner runs this binary once per bundle (crash isolation).
    if let [flag, bundle] = args.as_slice()
        && flag == "--scan-vst3"
    {
        std::process::exit(limeshot_ui::plugins::scan_child(bundle));
    }
    // Plugins set to run in a separate process run in a copy of this binary (crash isolation).
    #[cfg(unix)]
    if let Some(socket) = limeshot_plugins::bridge::socket_arg(&args) {
        std::process::exit(limeshot_plugins::bridge::child_main(
            &socket,
            &limeshot_plugins::load_module,
        ));
    }
    let samples = limeshot_app::bundled_samples_dir();
    let settings = Settings::load();
    let mut app = LimeShotApp::new(settings, samples.clone());
    app.plugins.bridge_exe = std::env::current_exe().ok();
    if args.iter().any(|a| a == "--demo") {
        match samples.as_deref().map(limeshot_app::demo::build) {
            Some(Ok(doc)) => {
                let project = doc.project().clone();
                app.doc.replace(project);
                if let Some(c) = app.project().channels.first().map(|c| c.id) {
                    app.select_channel(c);
                }
            }
            Some(Err(e)) => log::error!("demo: {e}"),
            None => log::error!("demo: bundled samples not found"),
        }
    }
    for a in &args {
        let (flag, name) = a.split_once('=').unwrap_or((a.as_str(), ""));
        let w = match name {
            "playlist" => Win::Playlist,
            "rack" => Win::ChannelRack,
            "piano" => Win::PianoRoll,
            "mixer" => Win::Mixer,
            "settings" => Win::ChannelSettings,
            _ => continue,
        };
        match flag {
            "--show" => app.show_window(w),
            "--detach" => {
                app.show_window(w);
                app.ui.workspace.state_mut(w).detached = true;
            }
            _ => {}
        }
    }
    if let Some(path) = args.iter().find(|a| !a.starts_with("--")) {
        app.open_path(std::path::Path::new(path));
    }
    app.offer_recovery();
    let icon = egui::IconData {
        rgba: limeshot_app::icon_rgba(256),
        width: 256,
        height: 256,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LimeShot")
            .with_app_id("org.limeshot.LimeShot")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 560.0])
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native("LimeShot", options, Box::new(|_cc| Ok(Box::new(App(app)))))
}
