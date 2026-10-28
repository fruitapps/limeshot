//! Write the LimeShot icon (drawn in code by `limeshot_app::icon_rgba`) for the packaging scripts:
//! * `make_icon -- dist/LimeShot.iconset`: a macOS iconset, then `iconutil -c icns dist/LimeShot.iconset`
//!   (`scripts/package-macos.sh`);
//! * `make_icon -- dist/LimeShot.ico`: a Windows icon with PNG-compressed 16…256 px images
//!   (`scripts/package-windows.ps1`);
//! * `make_icon -- dist/limeshot.png`: one 256 px PNG for Linux desktop files (`scripts/package-linux.sh`).

use std::path::Path;

fn main() {
    let Some(path) = std::env::args().nth(1).map(std::path::PathBuf::from) else {
        eprintln!("usage: make_icon <folder.iconset | file.ico | file.png>");
        std::process::exit(2);
    };
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let result = match ext.as_str() {
        "ico" => write_ico(&path),
        "png" => png(256).and_then(|bytes| write(&path, &bytes)),
        _ => write_iconset(&path),
    };
    if let Err(e) = result {
        eprintln!("{}: {e}", path.display());
        std::process::exit(1);
    }
    println!("wrote {}", path.display());
}

/// The icon at `px`×`px` as PNG bytes.
fn png(px: u32) -> Result<Vec<u8>, String> {
    let img = image::RgbaImage::from_raw(px, px, limeshot_app::icon_rgba(px))
        .ok_or_else(|| format!("icon size mismatch at {px}"))?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// The sizes iconutil expects: 16 … 512 points at 1x and 2x.
fn write_iconset(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for points in [16u32, 32, 128, 256, 512] {
        for scale in [1u32, 2] {
            let px = points * scale;
            let name = if scale == 1 {
                format!("icon_{points}x{points}.png")
            } else {
                format!("icon_{points}x{points}@2x.png")
            };
            write(&dir.join(&name), &png(px)?).map_err(|e| format!("{name}: {e}"))?;
        }
    }
    Ok(())
}

/// An ICO file: a 6-byte header, one 16-byte directory entry per image, then the images. Windows
/// Vista and later accept PNG data as the image, which keeps this free of a BMP encoder.
fn write_ico(path: &Path) -> Result<(), String> {
    const SIZES: [u32; 6] = [16, 24, 32, 48, 64, 256];
    let images = SIZES.iter().map(|&px| png(px)).collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]); // reserved, type 1 = icon
    out.extend_from_slice(&(SIZES.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * SIZES.len() as u32;
    for (&px, data) in SIZES.iter().zip(&images) {
        let dim = if px >= 256 { 0 } else { px as u8 }; // 0 means 256
        out.extend_from_slice(&[dim, dim, 0, 0]); // width, height, palette, reserved
        out.extend_from_slice(&1u16.to_le_bytes()); // colour planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += data.len() as u32;
    }
    for data in &images {
        out.extend_from_slice(data);
    }
    write(path, &out)
}

#[cfg(test)]
mod tests {
    /// Reject a bundled Windows icon that no longer matches the generated artwork.
    #[test]
    fn bundled_windows_icon_matches_generated_artwork() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("LimeShot.ico");
        super::write_ico(&path).unwrap();
        assert_eq!(
            std::fs::read(path).unwrap(),
            include_bytes!("../../../packaging/windows/LimeShot.ico"),
            "regenerate with cargo run -p limeshot-app --example make_icon -- packaging/windows/LimeShot.ico"
        );
    }
}
