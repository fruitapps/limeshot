//! Render all nine production editors offscreen without screen recording permissions.
use egui_kittest::Harness;
use lime_native::editor::{NativeEditor, editor_size, install_theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let folder = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "out/native-plugins".into());
    std::fs::create_dir_all(&folder)?;
    for id in 0..9 {
        let mut editor = if id == 1 {
            NativeEditor::preview_sampler(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/samples/Lime Snare.wav"),
            )?
        } else {
            NativeEditor::preview(id)
        };
        let (w, h) = editor_size(id);
        let mut initialized = false;
        let mut harness = Harness::builder().with_size(egui::vec2(w, h)).build_ui(move |ui| {
            if !initialized {
                install_theme(ui.ctx());
                initialized = true;
                return;
            }
            editor.draw(ui);
        });
        harness.run_steps(3);
        let path = folder.join(format!("{}.png", lime_native::parameters::NAMES[id as usize]));
        harness.render()?.save(&path)?;
        println!("{}", path.display());
    }
    Ok(())
}
