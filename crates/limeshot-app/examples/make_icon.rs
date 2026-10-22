//! Write the LimeShot icon (drawn in code by `limeshot_app::icon_rgba`) as a macOS iconset:
//! `cargo run -p limeshot-app --example make_icon -- dist/LimeShot.iconset`, then
//! `iconutil -c icns dist/LimeShot.iconset`. Used by `scripts/package-macos.sh`.

fn main() {
    let Some(dir) = std::env::args().nth(1).map(std::path::PathBuf::from) else {
        eprintln!("usage: make_icon <folder.iconset>");
        std::process::exit(2);
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("{}: {e}", dir.display());
        std::process::exit(1);
    }
    // The sizes iconutil expects: 16 … 512 points at 1x and 2x.
    for points in [16u32, 32, 128, 256, 512] {
        for scale in [1u32, 2] {
            let px = points * scale;
            let name = if scale == 1 {
                format!("icon_{points}x{points}.png")
            } else {
                format!("icon_{points}x{points}@2x.png")
            };
            let Some(img) = image::RgbaImage::from_raw(px, px, limeshot_app::icon_rgba(px)) else {
                eprintln!("icon size mismatch at {px}");
                std::process::exit(1);
            };
            if let Err(e) = img.save(dir.join(&name)) {
                eprintln!("{name}: {e}");
                std::process::exit(1);
            }
        }
    }
    println!("wrote {}", dir.display());
}
