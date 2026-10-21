//! Finding VST3 bundles in the standard folders and listing the plugins inside.
//!
//! [`scan_bundle`] loads the module into this process, so a broken plugin can crash it. The app
//! therefore scans each bundle in a child process (`limeshot --scan-vst3 <bundle>`) and keeps
//! the result in its plugin database; this module is what that child runs.

use std::path::{Path, PathBuf};

use crate::{PluginError, PluginInfo, Vst3Module};

/// The folders VST3 plugins are installed to on this platform, per the VST3 specification.
pub fn default_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/Library/Audio/Plug-Ins/VST3"));
        if let Some(h) = &home {
            dirs.push(h.join("Library/Audio/Plug-Ins/VST3"));
        }
    } else if cfg!(target_os = "windows") {
        let common = std::env::var_os("COMMONPROGRAMFILES")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files\Common Files"));
        dirs.push(common.join("VST3"));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join(r"Programs\Common\VST3"));
        }
    } else {
        if let Some(h) = &home {
            dirs.push(h.join(".vst3"));
        }
        dirs.push(PathBuf::from("/usr/lib/vst3"));
        dirs.push(PathBuf::from("/usr/local/lib/vst3"));
    }
    dirs
}

/// Every `.vst3` bundle (or single-file module) under `dirs`, sorted. Folders inside a bundle
/// are not searched.
pub fn find_bundles(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for d in dirs {
        walk(d, 0, &mut out);
    }
    out.sort();
    out.dedup();
    out
}

fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    // Vendors nest plugins in sub-folders; a bound keeps symlink loops finite.
    if depth > 8 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if is_bundle(&p) {
            out.push(p);
        } else if p.is_dir() {
            walk(&p, depth + 1, out);
        }
    }
}

pub fn is_bundle(p: &Path) -> bool {
    p.extension().is_some_and(|e| e.eq_ignore_ascii_case("vst3"))
}

/// Load the bundle at `path` and list its instruments and effects.
pub fn scan_bundle(path: &Path) -> Result<Vec<PluginInfo>, PluginError> {
    Ok(Vst3Module::load(path)?.classes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bundles_but_not_inside_them() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("A.vst3/Contents/MacOS");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(dir.path().join("A.vst3/Contents/Nested.vst3")).unwrap();
        std::fs::create_dir_all(dir.path().join("Vendor/B.VST3")).unwrap();
        std::fs::write(dir.path().join("C.vst3"), b"").unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"").unwrap();
        let found = find_bundles(&[dir.path().to_owned(), dir.path().join("missing")]);
        let names: Vec<_> = found
            .iter()
            .map(|p| p.strip_prefix(dir.path()).unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["A.vst3", "C.vst3", "Vendor/B.VST3"]);
    }

    #[test]
    fn a_missing_bundle_is_an_error() {
        assert!(scan_bundle(Path::new("/nonexistent/X.vst3")).is_err());
    }
}
