//! Shared code for the LimeShot binaries: locating bundled assets and building the demo song.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod demo;

use std::path::PathBuf;

/// The folder of bundled samples: next to the executable (`samples/`, or `../Resources/samples`
/// inside a macOS app bundle), falling back to the source tree in development builds.
pub fn bundled_samples_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()));
    let mut candidates = Vec::new();
    if let Some(d) = &exe_dir {
        candidates.push(d.join("samples"));
        candidates.push(d.join("../Resources/samples"));
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/samples"));
    candidates
        .into_iter()
        .find(|p| p.is_dir())
        .and_then(|p| p.canonicalize().ok())
}

/// The LimeShot icon as RGBA pixels, rasterised in code (no image file).
pub fn icon_rgba(size: u32) -> Vec<u8> {
    let mut px = vec![0u8; (size * size * 4) as usize];
    let c = size as f32 / 2.0;
    let r = size as f32 * 0.47;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - c;
            let dy = y as f32 + 0.5 - c;
            let d = (dx * dx + dy * dy).sqrt() / r;
            let a = dy.atan2(dx) + 0.2;
            let spoke = ((a / std::f32::consts::TAU * 8.0).fract() - 0.5).abs() < 0.04 && d > 0.12 && d < 0.8;
            let rgb = if d > 1.0 {
                None
            } else if d > 0.86 {
                Some([0x5E, 0x9E, 0x1E])
            } else if d > 0.78 || spoke || d < 0.14 {
                Some([0xEA, 0xF5, 0xC8])
            } else {
                Some([0xA6, 0xE0, 0x3A])
            };
            if let Some([rr, g, b]) = rgb {
                let i = ((y * size + x) * 4) as usize;
                let edge = ((1.0 - d) * r).clamp(0.0, 1.0);
                px[i..i + 4].copy_from_slice(&[rr, g, b, (edge * 255.0) as u8]);
            }
        }
    }
    px
}
