//! Autosave and crash recovery.
//!
//! While a project has unsaved changes, a copy is written every few minutes (Settings ›
//! `autosave_minutes`) to `autosave/autosave.lime` in the platform's data folder, next to a
//! `session.json` naming the file it came from. Saving, a clean New/Open and quitting remove it,
//! so a copy that is still there at the next start means LimeShot did not exit normally; the user
//! is then offered to recover it. Writes use the atomic project save.

use crate::app::LimeShotApp;
use limeshot_core::Project;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PROJECT_FILE: &str = "autosave.lime";
const SESSION_FILE: &str = "session.json";

/// What `session.json` records about the autosaved project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// The file the project was opened from or last saved to, if any.
    pub original: Option<PathBuf>,
    pub title: String,
    /// Seconds since the Unix epoch.
    pub saved_unix: u64,
}

impl SessionInfo {
    /// "3 minutes ago" style age of the autosave.
    pub fn age(&self) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let mins = now.saturating_sub(self.saved_unix) / 60;
        match mins {
            0 => "less than a minute ago".into(),
            1 => "1 minute ago".into(),
            m if m < 120 => format!("{m} minutes ago"),
            m => format!("{} hours ago", m / 60),
        }
    }
}

/// The default autosave folder (`<data dir>/autosave`).
pub fn default_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "LimeShot", "LimeShot").map(|d| d.data_dir().join("autosave"))
}

/// Autosave bookkeeping for one app instance.
#[derive(Debug, Default)]
pub struct Autosave {
    /// Where copies go; `None` disables autosave (tests, headless).
    pub dir: Option<PathBuf>,
    /// Overrides the interval from the settings (tests).
    pub interval: Option<Duration>,
    /// The document revision last written.
    written: Option<u64>,
    /// When the current run of unsaved changes began, or the last write.
    since: Option<Instant>,
    /// The copy on disk belongs to this session (written or recovered by it), so it may be
    /// removed once nothing is unsaved. A copy from a crashed session is left alone until the
    /// user recovers or discards it.
    owned: bool,
}

impl Autosave {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self {
            dir,
            interval: None,
            written: None,
            since: None,
            owned: false,
        }
    }

    /// Take over a recovered copy: it is removed once the recovered work is saved.
    pub fn adopt(&mut self) {
        self.owned = true;
    }

    /// A copy left behind by a session that did not end normally.
    pub fn find(&self) -> Option<SessionInfo> {
        let dir = self.dir.as_ref()?;
        if !dir.join(PROJECT_FILE).is_file() {
            return None;
        }
        let text = std::fs::read_to_string(dir.join(SESSION_FILE)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Load the autosaved project.
    pub fn load(&self) -> Result<Project, String> {
        let dir = self.dir.as_ref().ok_or("autosave is off")?;
        limeshot_project::load(&dir.join(PROJECT_FILE)).map_err(|e| e.to_string())
    }

    pub fn write(&mut self, project: &Project, original: Option<&Path>, revision: u64) -> Result<(), String> {
        let dir = self.dir.as_ref().ok_or("autosave is off")?;
        // Dismissing recovery does not grant ownership of the previous session’s copy.
        if !self.owned && dir.join(PROJECT_FILE).exists() {
            return Err("An earlier recovery copy exists; recover or discard it before autosaving this session".into());
        }
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        limeshot_project::save(project, &dir.join(PROJECT_FILE)).map_err(|e| e.to_string())?;
        let info = SessionInfo {
            original: original.map(Path::to_path_buf),
            title: project.title.clone(),
            saved_unix: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
        };
        let text = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(SESSION_FILE), text).map_err(|e| e.to_string())?;
        self.written = Some(revision);
        self.since = Some(Instant::now());
        self.owned = true;
        Ok(())
    }

    /// Normal saves/quits may remove only this session’s copy. Explicit Discard uses `clear`.
    pub(crate) fn clear_if_owned(&mut self) {
        if self.owned {
            self.clear();
        }
    }

    /// Remove the copy (the project was saved, replaced or LimeShot is quitting).
    pub fn clear(&mut self) {
        self.written = None;
        self.since = None;
        self.owned = false;
        if let Some(dir) = &self.dir {
            let _ = std::fs::remove_file(dir.join(PROJECT_FILE));
            let _ = std::fs::remove_file(dir.join(SESSION_FILE));
        }
    }
}

/// Called every frame: write a copy when unsaved changes are older than the interval, and remove
/// the copy once there is nothing unsaved.
pub fn tick(app: &mut LimeShotApp) {
    if app.autosave.dir.is_none() {
        return;
    }
    if !app.doc.is_modified() {
        if app.autosave.owned {
            app.autosave.clear();
        }
        app.autosave.since = None;
        return;
    }
    let interval = match app.autosave.interval {
        Some(i) => i,
        None if app.settings.autosave_minutes == 0 => return,
        None => Duration::from_secs(60 * u64::from(app.settings.autosave_minutes)),
    };
    let since = *app.autosave.since.get_or_insert_with(Instant::now);
    let revision = app.doc.revision();
    if app.autosave.written != Some(revision) && since.elapsed() >= interval {
        app.plugins.store_states(&mut app.doc);
        let (project, path) = (app.doc.project().clone(), app.path.clone());
        if let Err(e) = app.autosave.write(&project, path.as_deref(), revision) {
            app.toast_error(format!("Autosave failed: {e}"));
            // Try again after another interval rather than every frame.
            app.autosave.since = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn a_new_session_cannot_overwrite_or_clear_an_unclaimed_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let original = Project::default();
        let mut old = Autosave::new(Some(dir.path().into()));
        old.write(&original, None, 1).unwrap();
        let mut new = Autosave::new(Some(dir.path().into()));
        let mut replacement = original.clone();
        replacement.title = "Disposable test".into();
        assert!(new.write(&replacement, None, 2).is_err());
        new.clear_if_owned();
        assert_eq!(new.load().unwrap(), original);
        new.adopt();
        new.write(&replacement, None, 2).unwrap();
        assert_eq!(new.load().unwrap(), replacement);
        new.clear_if_owned();
        assert!(new.find().is_none());
    }
}
