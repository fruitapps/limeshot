//! Application settings (not part of any project), stored as JSON in the platform's config folder:
//! `~/Library/Application Support/org.LimeShot.LimeShot` on macOS, `%APPDATA%\LimeShot\LimeShot\config`
//! on Windows, `$XDG_CONFIG_HOME/limeshot` on Linux.

use limeshot_audio::AudioConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub audio: AudioConfig,
    pub recent: Vec<PathBuf>,
    /// Extra folders shown in the Browser.
    pub browser_folders: Vec<PathBuf>,
    /// Extra VST3 search roots, in addition to the platform defaults.
    pub vst3_folders: Vec<PathBuf>,
    pub ui_scale: f32,
    /// Options / toolbar: keep the active editor's playhead visible while playing.
    pub auto_scroll: bool,
    pub auto_keep_long_audio_on_disk: bool,
    /// Collapse channels without note data when switching patterns.
    pub auto_zip_empty_channels: bool,
    /// Minutes between autosaves of unsaved changes; 0 turns autosave off.
    pub autosave_minutes: u32,
    /// Ask before deleting channels (turned off by "Remember my choice").
    pub confirm_channel_delete: bool,
    /// View › Arrange windows › Save custom layout.
    pub custom_layout: Option<crate::workspace::SavedLayout>,
    /// The MIDI input device whose notes play the selected channel (FL Studio's MIDI settings);
    /// `None`: no MIDI input.
    pub midi_input: Option<String>,
    /// FL Studio's "Link note on velocity to": the key's velocity, or a fixed one ("Nothing").
    pub midi_velocity: bool,
    /// The audio input recorded from (`None`: the system default input).
    pub audio_input: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            recent: Vec::new(),
            browser_folders: Vec::new(),
            vst3_folders: Vec::new(),
            ui_scale: 1.0,
            auto_scroll: false,
            auto_keep_long_audio_on_disk: true,
            auto_zip_empty_channels: false,
            autosave_minutes: 5,
            confirm_channel_delete: true,
            custom_layout: None,
            midi_input: None,
            midi_velocity: true,
            audio_input: None,
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "LimeShot", "LimeShot").map(|d| d.config_dir().to_path_buf())
}

fn settings_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

impl Settings {
    /// Load from the config folder; defaults when missing or unreadable.
    pub fn load() -> Self {
        settings_file().map(|f| Self::load_from(&f)).unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str::<Settings>(&t).ok())
            .map(|mut s| {
                s.ui_scale = if s.ui_scale.is_finite() {
                    s.ui_scale.clamp(0.5, 3.0)
                } else {
                    1.0
                };
                s.recent.truncate(MAX_RECENT);
                s.autosave_minutes = s.autosave_minutes.min(60);
                s
            })
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = settings_file() else { return };
        if let Err(e) = self.save_to(&path) {
            log::warn!("could not save settings: {e}");
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(tmp, path)
    }

    pub fn add_recent(&mut self, path: &Path) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(MAX_RECENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_recent_list() {
        let dir = tempfile::tempdir().expect("tempdir");
        let f = dir.path().join("s.json");
        let mut s = Settings::default();
        assert!(
            !serde_json::from_str::<Settings>("{}")
                .expect("old settings")
                .auto_scroll
        );
        for i in 0..15 {
            s.add_recent(Path::new(&format!("/p/{i}.lime")));
        }
        s.add_recent(Path::new("/p/3.lime"));
        assert_eq!(s.recent.len(), MAX_RECENT);
        assert_eq!(s.recent[0], Path::new("/p/3.lime"));
        assert!(
            !serde_json::from_str::<Settings>("{}")
                .expect("old settings")
                .auto_zip_empty_channels
        );
        assert!(serde_json::from_str::<Settings>("{}").unwrap().vst3_folders.is_empty());
        s.vst3_folders = vec![dir.path().join("Custom VST3"), dir.path().join("Missing")];
        s.auto_scroll = true;
        s.auto_zip_empty_channels = true;
        s.save_to(&f).expect("save");
        assert_eq!(Settings::load_from(&f), s);
        std::fs::write(&f, "garbage").expect("write");
        assert_eq!(Settings::load_from(&f), Settings::default());
    }
}
