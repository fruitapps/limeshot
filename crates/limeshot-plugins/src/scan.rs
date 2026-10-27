//! Finding VST3 bundles in the standard folders and listing the plugins inside.
//!
//! [`scan_bundle`] loads the module into this process, so a broken plugin can crash it. The app
//! therefore scans each bundle in a child process (`limeshot --scan-vst3 <bundle>`) and keeps
//! the result in its plugin database; this module is what that child runs.

use std::collections::{HashMap, HashSet};
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
    let mut visited = HashMap::new();
    let mut bundles = HashSet::new();
    for d in dirs {
        walk(d, 0, &mut visited, &mut bundles, &mut out);
    }
    out.sort();
    out
}

fn walk(
    path: &Path,
    depth: usize,
    visited: &mut HashMap<PathBuf, usize>,
    bundles: &mut HashSet<PathBuf>,
    out: &mut Vec<PathBuf>,
) {
    // Canonical identities avoid rescanning overlapping roots, aliases and symlink cycles.
    // Load the canonical bundle: Linux/Windows derive its binary name from the bundle stem,
    // so a renamed symlink cannot be used as the representative path.
    let Ok(identity) = std::fs::canonicalize(path) else {
        return;
    };
    if is_bundle(path) {
        if bundles.insert(identity.clone()) {
            out.push(identity);
        }
        return;
    }
    if depth > 8
        || visited
            .get(&identity)
            .is_some_and(|previous_depth| *previous_depth <= depth)
    {
        return;
    }
    visited.insert(identity, depth);
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for e in entries.flatten() {
        walk(&e.path(), depth + 1, visited, bundles, out);
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
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let a = dir.as_path().join("A.vst3/Contents/MacOS");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(dir.as_path().join("A.vst3/Contents/Nested.vst3")).unwrap();
        std::fs::create_dir_all(dir.as_path().join("Vendor/B.VST3")).unwrap();
        std::fs::write(dir.as_path().join("C.vst3"), b"").unwrap();
        std::fs::write(dir.as_path().join("readme.txt"), b"").unwrap();
        let found = find_bundles(&[dir.as_path().to_owned(), dir.as_path().join("missing")]);
        let names: Vec<_> = found
            .iter()
            .map(|p| p.strip_prefix(dir.as_path()).unwrap().to_path_buf())
            .collect();
        assert_eq!(
            names,
            [
                PathBuf::from("A.vst3"),
                PathBuf::from("C.vst3"),
                PathBuf::from("Vendor/B.VST3")
            ]
        );
    }

    #[test]
    fn overlapping_custom_roots_and_bundle_roots_scan_once() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let vendor = dir.as_path().join("Vendor");
        let bundle = vendor.join("Custom.vst3");
        std::fs::create_dir_all(bundle.join("Contents/Hidden.vst3")).unwrap();
        let roots = [
            dir.as_path().to_owned(),
            vendor,
            bundle.clone(),
            dir.as_path().to_owned(),
        ];
        assert_eq!(find_bundles(&roots), std::slice::from_ref(&bundle));
        assert_eq!(find_bundles(std::slice::from_ref(&bundle)), [bundle]);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_aliases_and_cycles_do_not_duplicate_bundles() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let root = dir.as_path().join("Plugins");
        let bundle = root.join("Custom.vst3");
        std::fs::create_dir_all(&bundle).unwrap();
        std::os::unix::fs::symlink(&root, root.join("Cycle")).unwrap();
        let alias = dir.as_path().join("Alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        assert_eq!(find_bundles(&[root, alias]), std::slice::from_ref(&bundle));
        let bundle_alias = dir.as_path().join("Renamed.vst3");
        std::os::unix::fs::symlink(&bundle, &bundle_alias).unwrap();
        assert_eq!(find_bundles(&[bundle_alias, bundle.clone()]), [bundle]);
    }

    #[test]
    fn a_shallower_extra_root_can_reach_beyond_an_earlier_depth_limit() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().canonicalize().unwrap();
        let deep = dir.as_path().join("1/2/3/4/5/6/7/8");
        let bundle = deep.join("9/Custom.vst3");
        std::fs::create_dir_all(&bundle).unwrap();
        assert_eq!(find_bundles(&[dir.as_path().to_owned(), deep]), [bundle]);
    }

    #[test]
    fn a_missing_bundle_is_an_error() {
        assert!(scan_bundle(Path::new("/nonexistent/X.vst3")).is_err());
    }
}
