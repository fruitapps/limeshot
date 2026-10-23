//! Where the Lime Native VST3 plugins (LimeShot's own instruments and effects) are found.
//!
//! Projects refer to them as `lime:<bundle>` (see `limeshot_core::lime`), so a project does not
//! depend on where LimeShot is installed. [`bundle_path`] looks, in order, in the folder named by
//! `LIMESHOT_LIME_PLUGINS`, next to the executable (`../PlugIns` inside a macOS app, `plugins`
//! beside it elsewhere), in this checkout's `native-plugins/dist` (development builds), and in the
//! platform's standard VST3 folders.

use std::path::PathBuf;

use crate::scan;

/// The prefix of a Lime plugin's path in a project.
pub const PATH_PREFIX: &str = "lime:";

/// The folders searched for Lime bundles, in order.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = std::env::var_os("LIMESHOT_LIME_PLUGINS") {
        dirs.push(PathBuf::from(d));
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(PathBuf::from)) {
        dirs.push(exe_dir.join("../PlugIns"));
        dirs.push(exe_dir.join("plugins"));
    }
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../native-plugins/dist"));
    dirs.extend(scan::default_dirs());
    dirs
}

/// The installed bundle for a `lime:` path, or `None` for other paths or when it is not found.
pub fn bundle_path(path: &str) -> Option<PathBuf> {
    let bundle = path.strip_prefix(PATH_PREFIX)?;
    search_dirs().into_iter().map(|d| d.join(bundle)).find(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_override_folder_comes_first() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("LimeSynth.vst3")).unwrap();
        // Only this test sets the variable.
        unsafe { std::env::set_var("LIMESHOT_LIME_PLUGINS", dir.path()) };
        assert_eq!(
            bundle_path("lime:LimeSynth.vst3"),
            Some(dir.path().join("LimeSynth.vst3"))
        );
        assert_eq!(bundle_path("/x/LimeSynth.vst3"), None);
        unsafe { std::env::remove_var("LIMESHOT_LIME_PLUGINS") };
    }
}
