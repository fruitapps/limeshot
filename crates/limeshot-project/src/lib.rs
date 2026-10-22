//! `.lime` project files.
//!
//! A `.lime` file is UTF-8 JSON with a versioned envelope:
//!
//! ```json
//! { "format": "limeshot-project", "version": 1, "generator": "LimeShot 0.1.0", "project": { … } }
//! ```
//!
//! * Files from older versions are migrated step by step ([`migrate`]).
//! * Files from newer versions are refused rather than half-read.
//! * Unknown fields inside the project are kept (see `limeshot_core::model::Extra`).
//! * Sample paths are stored relative to the project file when the sample is inside the project
//!   folder, absolute otherwise; [`resolve_sample_path`] turns them back into paths.
//! * Saving writes a temporary file next to the target, syncs it and renames it over the target,
//!   so a crash mid-save never leaves a truncated project.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod flp;
pub mod midi;
pub mod preset;

use limeshot_core::{Instrument, Project};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "limeshot-project";
pub const CURRENT_VERSION: u64 = 1;
pub const EXTENSION: &str = "lime";
/// Larger files are refused (a project is notes and settings, not audio).
pub const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("could not read or write {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("{0} is not a LimeShot project")]
    NotAProject(PathBuf),
    #[error(
        "{path} was saved by a newer LimeShot (format version {version}); this version reads up to {CURRENT_VERSION}"
    )]
    TooNew { path: PathBuf, version: u64 },
    #[error("{path} is damaged: {message}")]
    Corrupt { path: PathBuf, message: String },
    #[error("{0} is larger than the 256 MB project limit")]
    TooLarge(PathBuf),
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> ProjectError + '_ {
    move |source| ProjectError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Serialise `project` to the envelope JSON. Sample paths are made relative to `project_dir`.
pub fn to_json(project: &Project, project_dir: Option<&Path>) -> Result<String, serde_json::Error> {
    let mut p = project.clone();
    if let Some(dir) = project_dir {
        for c in &mut p.channels {
            if let Instrument::Sampler(s) = &mut c.instrument
                && let Some(r) = &mut s.sample
            {
                r.path = relative_path(Path::new(&r.path), dir);
            }
        }
    }
    let env = json!({
        "format": FORMAT,
        "version": CURRENT_VERSION,
        "generator": concat!("LimeShot ", env!("CARGO_PKG_VERSION")),
        "project": p,
    });
    serde_json::to_string_pretty(&env)
}

/// Parse envelope JSON (migrating old versions) into a sanitized project. Sample paths stay as
/// stored; use [`resolve_sample_path`] to locate them.
pub fn from_json(text: &str, path_for_errors: &Path) -> Result<Project, ProjectError> {
    let corrupt = |message: String| ProjectError::Corrupt {
        path: path_for_errors.to_path_buf(),
        message,
    };
    let env: Value = serde_json::from_str(text).map_err(|e| corrupt(e.to_string()))?;
    if env.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(ProjectError::NotAProject(path_for_errors.to_path_buf()));
    }
    let version = env
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| corrupt("missing format version".into()))?;
    if version > CURRENT_VERSION {
        return Err(ProjectError::TooNew {
            path: path_for_errors.to_path_buf(),
            version,
        });
    }
    let raw = env
        .get("project")
        .cloned()
        .ok_or_else(|| corrupt("missing project data".into()))?;
    let raw = migrate(version, raw).map_err(corrupt)?;
    let mut project: Project = serde_json::from_value(raw).map_err(|e| corrupt(e.to_string()))?;
    project.sanitize();
    Ok(project)
}

/// Bring project JSON from `version` up to [`CURRENT_VERSION`]. Each future format change adds a
/// step here; version 1 is the first format.
pub fn migrate(version: u64, project: Value) -> Result<Value, String> {
    match version {
        0 => Err("format version 0 does not exist".into()),
        CURRENT_VERSION => Ok(project),
        v => Err(format!("no migration from format version {v}")),
    }
}

pub fn load(path: &Path) -> Result<Project, ProjectError> {
    let meta = fs::metadata(path).map_err(io(path))?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(ProjectError::TooLarge(path.to_path_buf()));
    }
    let text = fs::read_to_string(path).map_err(io(path))?;
    from_json(&text, path)
}

/// Save atomically: write `name.lime.tmp-<pid>`, flush to disk, rename over `path`.
pub fn save(project: &Project, path: &Path) -> Result<(), ProjectError> {
    let dir = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let text = to_json(project, Some(dir)).map_err(|e| ProjectError::Corrupt {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project.lime".into());
    let tmp = dir.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut f = fs::File::create(&tmp).map_err(io(&tmp))?;
        f.write_all(text.as_bytes()).map_err(io(&tmp))?;
        f.sync_all().map_err(io(&tmp))?;
        drop(f);
        fs::rename(&tmp, path).map_err(io(path))?;
        // Make the rename itself durable where the platform allows syncing a directory.
        #[cfg(unix)]
        if let Ok(d) = fs::File::open(dir) {
            let _ = d.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// `target` relative to `dir` when it lies inside `dir`, otherwise unchanged. Separators are
/// written as `/` so files move between operating systems.
pub fn relative_path(target: &Path, dir: &Path) -> String {
    if target.is_relative() {
        return target.to_string_lossy().replace('\\', "/");
    }
    match target.strip_prefix(dir) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
        Err(_) => target.to_string_lossy().into_owned(),
    }
}

/// Where a stored sample path points: relative paths are resolved against the project folder
/// (if known), absolute paths are used as they are.
pub fn resolve_sample_path(stored: &str, project_dir: Option<&Path>) -> PathBuf {
    let p = Path::new(stored);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let native: PathBuf = stored.split('/').collect();
    match project_dir {
        Some(d) => d.join(native),
        None => native,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use limeshot_core::*;

    fn sample_project(sample: &str) -> Project {
        let mut d = Document::default();
        let pat = d.project().patterns[0].id;
        let kick = ChannelId(
            d.execute(Command::AddChannel {
                name: "Kick".into(),
                instrument: Instrument::Sampler(SamplerParams {
                    sample: Some(SampleRef { path: sample.into() }),
                    ..SamplerParams::default()
                }),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let lead = ChannelId(
            d.execute(Command::AddChannel {
                name: "Lead".into(),
                instrument: Instrument::Synth(SynthParams::default()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        for s in [0, 4, 8, 12] {
            d.execute(Command::SetStep {
                pattern: pat,
                channel: kick,
                step: s,
                on: true,
            })
            .unwrap();
        }
        d.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: lead,
                key: 67,
                start: 480,
                length: 720,
                velocity: 0.6,
            },
        })
        .unwrap();
        d.execute(Command::AddClip {
            pattern: pat,
            track: 0,
            start: 0,
            length: None,
        })
        .unwrap();
        d.execute(Command::SetTempo(Tempo::from_bpm(97.25))).unwrap();
        d.execute(Command::SetMixerVolume { track: 1, volume: 0.5 }).unwrap();
        d.execute(Command::SetChannelPan {
            channel: lead,
            pan: -0.25,
        })
        .unwrap();
        d.project().clone()
    }

    #[test]
    fn save_and_reload_is_identical() {
        let dir = tempfile::tempdir().unwrap();
        let sample = dir.path().join("samples").join("kick.wav");
        let project = sample_project(&sample.to_string_lossy());
        let path = dir.path().join("song.lime");
        save(&project, &path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("\"path\": \"samples/kick.wav\""),
            "sample stored relative: {text}"
        );
        let loaded = load(&path).unwrap();
        let mut expected = project.clone();
        if let Instrument::Sampler(s) = &mut expected.channels[0].instrument {
            s.sample = Some(SampleRef {
                path: "samples/kick.wav".into(),
            });
        }
        assert_eq!(loaded, expected);
        assert_eq!(resolve_sample_path("samples/kick.wav", Some(dir.path())), sample);
        // No temporary files left behind.
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names.len(), 1, "{names:?}");
    }

    #[test]
    fn unknown_fields_survive() {
        let project = sample_project("/abs/kick.wav");
        let mut v: Value = serde_json::from_str(&to_json(&project, None).unwrap()).unwrap();
        v["project"]["future_feature"] = json!({"a": 1});
        v["project"]["channels"][0]["sidechain"] = json!(true);
        let p = from_json(&v.to_string(), Path::new("x.lime")).unwrap();
        let again: Value = serde_json::from_str(&to_json(&p, None).unwrap()).unwrap();
        assert_eq!(again["project"]["future_feature"], json!({"a": 1}));
        assert_eq!(again["project"]["channels"][0]["sidechain"], json!(true));
        assert_eq!(
            again["project"]["channels"][0]["instrument"]["sample"]["path"],
            json!("/abs/kick.wav")
        );
    }

    #[test]
    fn refuses_newer_and_foreign_files() {
        let e = from_json(
            r#"{"format":"limeshot-project","version":99,"project":{}}"#,
            Path::new("a.lime"),
        );
        assert!(matches!(e, Err(ProjectError::TooNew { version: 99, .. })));
        let e = from_json(r#"{"hello":1}"#, Path::new("a.lime"));
        assert!(matches!(e, Err(ProjectError::NotAProject(_))));
        let e = from_json("{not json", Path::new("a.lime"));
        assert!(matches!(e, Err(ProjectError::Corrupt { .. })));
    }

    #[test]
    fn overwriting_keeps_one_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.lime");
        save(&Project::new(), &path).unwrap();
        let mut p = Project::new();
        p.title = "Second".into();
        save(&p, &path).unwrap();
        assert_eq!(load(&path).unwrap().title, "Second");
    }
}
