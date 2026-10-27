//! LimeShot's instruments and effects are the Lime Native VST3 plugins (decision D39). The app
//! installs [`normalizer`] on its document, so former built-in instruments and effects, whether
//! opened from a project or created by an edit, become Lime plugins before they reach the engine.

use std::path::Path;

use limeshot_core::{EmbeddedAudio, Project, lime};

/// Decode a sampler's sample (an absolute path in memory) for Lime Sampler's state.
pub fn embedded_audio(stored: &str) -> Option<EmbeddedAudio> {
    let path = limeshot_project::resolve_sample_path(stored, None);
    let data = limeshot_dsp::load_audio_bounded(&path, lime::MAX_EMBEDDED_FRAMES).ok()?;
    Some(EmbeddedAudio {
        name: Path::new(stored)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        sample_rate: data.sample_rate,
        stereo: data.data,
    })
}

/// Opening a project may resolve references, but must not silently copy unrelated local recordings
/// into portable state. Canonical paths also catch symlinks escaping the project folder.
pub fn restrict_loaded_sources(project: &mut Project, directory: Option<&Path>) {
    let root = directory
        .map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p })
        .and_then(|p| p.canonicalize().ok());
    let is_inside = |raw: &str| {
        let source = limeshot_project::resolve_sample_path(raw, directory)
            .canonicalize()
            .ok();
        root.as_ref()
            .zip(source.as_ref())
            .is_some_and(|(root, source)| source.starts_with(root))
    };
    for channel in &mut project.channels {
        let (has_source, inside, unapproved_disk) = match &channel.instrument {
            limeshot_core::Instrument::Sampler(p) => match &p.sample {
                Some(s) => (true, is_inside(&s.path), false),
                None => (false, true, false),
            },
            limeshot_core::Instrument::Plugin(p) => {
                let disk = lime::disk_source(p);
                let pending = (p.extra.get("lime-sample-pending").and_then(serde_json::Value::as_bool) == Some(true))
                    .then(|| {
                        p.extra
                            .get("lime-sample-path")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_owned)
                    })
                    .flatten();
                let disk_ok = disk.as_deref().is_none_or(is_inside);
                let pending_ok = pending.as_deref().is_none_or(is_inside);
                (disk.is_some() || pending.is_some(), disk_ok && pending_ok, !disk_ok)
            }
            _ => (false, true, false),
        };
        if has_source {
            if inside {
                channel.extra.remove("lime-external-source");
            } else {
                channel
                    .extra
                    .insert("lime-external-source".into(), serde_json::json!(true));
            }
            if unapproved_disk && let limeshot_core::Instrument::Plugin(plugin) = &mut channel.instrument {
                plugin
                    .extra
                    .insert("lime-disk-unapproved".into(), serde_json::json!(true));
            }
        }
    }
}

/// The document normalizer that converts to Lime plugins. Failed source decodes are cached by
/// path and file metadata so unrelated edits do not repeatedly decode unchanged sources.
pub fn normalizer() -> limeshot_core::document::Normalizer {
    normalizer_with_audio(embedded_audio)
}

fn normalizer_with_audio(
    audio: impl Fn(&str) -> Option<EmbeddedAudio> + Send + 'static,
) -> limeshot_core::document::Normalizer {
    let failed = std::cell::RefCell::new(std::collections::HashMap::new());
    Box::new(move |project: &mut Project| {
        if lime::needs_conversion(project) {
            // New legacy sources need their first conversion and its precise failure warning.
            let fresh: std::collections::HashSet<_> = project
                .channels
                .iter()
                .filter_map(|c| {
                    if let limeshot_core::Instrument::Sampler(p) = &c.instrument {
                        p.sample.as_ref().map(|s| s.path.clone())
                    } else {
                        None
                    }
                })
                .collect();
            lime::convert_project(project, &|source| {
                let path = limeshot_project::resolve_sample_path(source, None);
                let version = std::fs::metadata(&path).ok().map(|m| (m.len(), m.modified().ok()));
                if !fresh.contains(source) && failed.borrow().get(&path) == Some(&version) {
                    return None;
                }
                let sample = audio(source);
                // Retain only failed attempts, never decoded PCM. Restored or changed files retry.
                if sample.as_ref().is_none_or(|s| !lime::embedded_audio_supported(s)) {
                    failed.borrow_mut().insert(path, version);
                } else {
                    failed.borrow_mut().remove(&path);
                }
                sample
            })
        } else {
            Vec::new()
        }
    })
}

/// Decode only on document changes, never from drawing or the audio callback. A content key
/// shares cached PCM across parameter edits and channels, even after the source file goes away.
pub fn cache_embedded_sample(
    state: &limeshot_core::PluginState,
    cache: &mut limeshot_audio::assets::SampleCache,
) -> Option<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    let component: serde_json::Value = serde_json::from_slice(lime::component_state(state)?).ok()?;
    let text = component["fields"]["sample-v1"].as_str()?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hash);
    let path = std::path::PathBuf::from(format!("/.limeshot-embedded/{:016x}.wav", hash.finish()));
    if cache.status_of_path(&path).is_some() {
        return Some(path);
    }
    let sample: serde_json::Value = serde_json::from_str(text).ok()?;
    let rate = u32::try_from(sample["sample_rate"].as_u64()?).ok()?;
    if sample["keep_on_disk"].as_bool() == Some(true) {
        let source = std::path::PathBuf::from(sample["source"].as_str()?);
        cache.request_disk_path(source.clone());
        return Some(source);
    }
    let pcm = sample["pcm"].as_str()?;
    if !(8000..=384000).contains(&rate) || pcm.len() > (lime::MAX_EMBEDDED_FRAMES * 8).div_ceil(3) * 4 {
        return None;
    }
    let bytes = limeshot_core::plugin::base64_decode(pcm)?;
    if bytes.len() < 16 || bytes.len() % 8 != 0 {
        return None;
    }
    let data = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| {
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            if v.is_finite() { v } else { 0.0 }
        })
        .collect();
    if !cache.insert_embedded(
        path.clone(),
        std::sync::Arc::new(limeshot_dsp::SampleData::from_stereo(rate, data)),
    ) {
        return None;
    }
    Some(path)
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use limeshot_core::*;

    fn sampler(path: &Path) -> Project {
        let mut doc = Document::default();
        doc.execute(Command::AddChannel {
            name: "Sample".into(),
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: path.to_string_lossy().into_owned(),
                }),
                ..Default::default()
            }),
        })
        .unwrap();
        doc.project().clone()
    }

    #[test]
    fn disk_state_source_is_checked_even_with_missing_or_misleading_metadata() {
        let inside = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let local = inside.path().join("local.wav");
        let external = outside.path().join("private.wav");
        std::fs::write(&local, []).unwrap();
        std::fs::write(&external, []).unwrap();
        for metadata in [None, Some(local.to_string_lossy().into_owned())] {
            let mut plugin =
                LimePlugin::SAMPLER.with_values(&lime::sampler_values(&SamplerParams::default(), 60), false, None);
            assert!(lime::set_disk_source(&mut plugin, &external.to_string_lossy()));
            if let Some(metadata) = metadata {
                plugin
                    .extra
                    .insert("lime-sample-path".into(), serde_json::json!(metadata));
            }
            let mut doc = Document::default();
            doc.execute(Command::AddChannel {
                name: "Disk".into(),
                instrument: Instrument::Plugin(plugin),
            })
            .unwrap();
            let mut project = doc.project().clone();
            restrict_loaded_sources(&mut project, Some(inside.path()));
            assert_eq!(project.channels[0].extra["lime-external-source"], true);
            let Instrument::Plugin(plugin) = &project.channels[0].instrument else {
                panic!("plugin")
            };
            assert_eq!(plugin.extra["lime-disk-unapproved"], true);
        }
    }

    #[test]
    fn project_open_requires_explicit_selection_before_embedding_external_audio() {
        let inside = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let local = inside.path().join("local.wav");
        let external = outside.path().join("private.wav");
        for path in [&local, &external] {
            limeshot_dsp::write_wav(path, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
        }
        let mut project = sampler(&external);
        restrict_loaded_sources(&mut project, Some(inside.path()));
        let warnings = lime::convert_project(&mut project, &embedded_audio);
        assert!(warnings.iter().any(|w| w.contains("outside-project")));
        let channel = project.channels[0].id;
        let Instrument::Plugin(p) = &project.channels[0].instrument else {
            panic!("not hosted")
        };
        assert_eq!(p.extra["lime-sample-pending"], true);
        let state: serde_json::Value = serde_json::from_slice(lime::component_state(&p.state).unwrap()).unwrap();
        assert!(state["fields"].get("sample-v1").is_none());
        assert!(crate::plugins::PluginHost::new(None).offline(&project, 48000).is_err());
        let roundtrip: Project = serde_json::from_str(&serde_json::to_string(&project).unwrap()).unwrap();
        let mut doc = Document::new(roundtrip);
        doc.set_normalizer(normalizer());
        doc.execute(Command::SetInstrument {
            channel,
            instrument: Instrument::Sampler(SamplerParams {
                sample: Some(SampleRef {
                    path: external.to_string_lossy().into_owned(),
                }),
                ..Default::default()
            }),
        })
        .unwrap();
        assert_eq!(
            doc.project().channels[0].extra["lime-external-source"],
            true,
            "replacing preset/controls does not approve an external file"
        );
        assert_eq!(
            doc.project()
                .plugin(limeshot_core::PluginTarget::Channel(channel))
                .unwrap()
                .extra["lime-sample-pending"],
            true
        );
        doc.execute(Command::ApproveSamplerSource { channel }).unwrap();
        let Instrument::Plugin(p) = &doc.project().channels[0].instrument else {
            panic!("not hosted")
        };
        assert!(p.extra.get("lime-sample-pending").is_none());
        assert!(cache_embedded_sample(&p.state, &mut limeshot_audio::assets::SampleCache::new()).is_some());
        assert!(doc.undo());
        assert_eq!(doc.project().channels[0].extra["lime-external-source"], true);
        let mut project = sampler(&local);
        restrict_loaded_sources(&mut project, Some(inside.path()));
        assert!(lime::convert_project(&mut project, &embedded_audio).is_empty());
        // A bare CLI filename has an empty parent path, meaning the current project folder.
        let relative = tempfile::NamedTempFile::new_in(".").unwrap();
        let mut project = sampler(relative.path());
        restrict_loaded_sources(&mut project, Some(Path::new("")));
        assert!(project.channels[0].extra.get("lime-external-source").is_none());
    }

    #[test]
    fn missing_sample_conversion_keeps_a_retry_route_across_save_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("late.wav");
        let mut project = sampler(&path);
        assert!(!lime::convert_project(&mut project, &embedded_audio).is_empty());
        let mut project: Project = serde_json::from_str(&serde_json::to_string(&project).unwrap()).unwrap();
        let original = project.channels[0].instrument.clone();
        assert!(!lime::convert_project(&mut project, &embedded_audio).is_empty());
        assert_eq!(project.channels[0].instrument, original);
        limeshot_dsp::write_wav(&path, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
        restrict_loaded_sources(&mut project, Some(dir.path()));
        assert!(lime::convert_project(&mut project, &embedded_audio).is_empty());
        assert!(!lime::needs_conversion(&project));
    }

    #[test]
    fn unrelated_edits_do_not_decode_unchanged_failed_sources() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        // Both decoder failures and audio rejected by the native sampler must be cached.
        for unsupported in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("source.wav");
            if unsupported {
                limeshot_dsp::write_wav(&path, 4000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
            }
            let attempts = Arc::new(AtomicUsize::new(0));
            let loaded = attempts.clone();
            let mut doc = Document::new(sampler(&path));
            doc.execute(Command::AddChannel {
                name: "Same source".into(),
                instrument: doc.project().channels[0].instrument.clone(),
            })
            .unwrap();
            doc.set_normalizer(normalizer_with_audio(move |source| {
                loaded.fetch_add(1, Ordering::SeqCst);
                embedded_audio(source)
            }));
            assert_eq!(attempts.load(Ordering::SeqCst), 2);
            if unsupported {
                assert_eq!(
                    doc.take_warnings()
                        .iter()
                        .filter(|w| w.contains("state limits"))
                        .count(),
                    2
                );
            }
            for tempo in [125_000, 126_000, 127_000] {
                doc.execute(Command::SetTempo(Tempo::from_milli_bpm(tempo))).unwrap();
            }
            assert_eq!(attempts.load(Ordering::SeqCst), 2);
            assert!(lime::needs_conversion(doc.project()));
            assert_eq!(
                lime::sampler_params(&doc.project().channels[0].instrument)
                    .unwrap()
                    .sample
                    .unwrap()
                    .path,
                path.to_string_lossy()
            );

            // Replacing the source without changing its length still retries on modified time.
            limeshot_dsp::write_wav(&path, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
            let modified = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(modified))
                .unwrap();
            doc.execute(Command::SetTempo(Tempo::from_milli_bpm(128_000))).unwrap();
            assert_eq!(attempts.load(Ordering::SeqCst), 4);
            assert!(!lime::needs_conversion(doc.project()));
            doc.execute(Command::SetTempo(Tempo::from_milli_bpm(129_000))).unwrap();
            assert_eq!(attempts.load(Ordering::SeqCst), 4);
        }
    }

    #[test]
    #[cfg(unix)]
    fn project_sample_symlinks_cannot_embed_files_outside_the_project() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let audio = outside.path().join("private.wav");
        limeshot_dsp::write_wav(&audio, 48000, &[0.5; 4], limeshot_dsp::BitDepth::Float32).unwrap();
        let link = dir.path().join("alias.wav");
        std::os::unix::fs::symlink(&audio, &link).unwrap();
        let mut project = sampler(&link);
        restrict_loaded_sources(&mut project, Some(dir.path()));
        assert_eq!(project.channels[0].extra["lime-external-source"], true);
    }
}
