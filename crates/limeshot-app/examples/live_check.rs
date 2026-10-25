//! Launch the real desktop UI with a disposable project, without shared autosave or recovery.
//! `LIMESHOT_LIME_PLUGINS=... cargo run -p limeshot-app --example live_check -- /tmp/check.lime`

struct LiveCheck(limeshot_ui::LimeShotApp);

impl eframe::App for LiveCheck {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}

fn main() -> eframe::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut app = limeshot_ui::LimeShotApp::new(limeshot_ui::Settings::default(), limeshot_app::bundled_samples_dir());
    app.autosave.dir = None;
    app.persist_settings = false;
    if let Some(path) = std::env::args_os().nth(1) {
        app.open_path(std::path::Path::new(&path));
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LimeShot Live Check")
            .with_app_id("org.limeshot.livecheck")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LimeShot Live Check",
        options,
        Box::new(|_cc| Ok(Box::new(LiveCheck(app)))),
    )
}
