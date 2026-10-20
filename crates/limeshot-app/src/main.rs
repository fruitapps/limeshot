//! The LimeShot desktop application.
//!
//! `limeshot [project.lime] [--demo]`

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
    let samples = limeshot_app::bundled_samples_dir();
    let settings = Settings::load();
    let mut app = LimeShotApp::new(settings, samples.clone());
    if args.iter().any(|a| a == "--demo") {
        match samples.as_deref().map(limeshot_app::demo::build) {
            Some(Ok(doc)) => {
                let project = doc.project().clone();
                app.doc.replace(project);
                app.ui.selected_channel = app.project().channels.first().map(|c| c.id);
            }
            Some(Err(e)) => log::error!("demo: {e}"),
            None => log::error!("demo: bundled samples not found"),
        }
    }
    if let Some(path) = args.iter().find(|a| !a.starts_with("--")) {
        app.open_path(std::path::Path::new(path));
    }
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
