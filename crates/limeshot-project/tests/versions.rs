//! Numbered saves must never replace an older project, including concurrent saves.
use limeshot_core::*;
use limeshot_project::{load, save, save_new_version};

#[test]
fn numbered_saves_skip_files_and_directories_and_keep_sample_paths() {
    let dir = tempfile::tempdir().unwrap();
    let sample = dir.path().join("samples/kick.wav");
    let mut doc = Document::default();
    doc.execute(Command::AddChannel {
        name: "Kick".into(),
        instrument: Instrument::Sampler(SamplerParams {
            sample: Some(SampleRef {
                path: sample.to_string_lossy().into_owned(),
            }),
            ..SamplerParams::default()
        }),
    })
    .unwrap();
    let original = dir.path().join("My song.lime");
    save(doc.project(), &original).unwrap();
    let original_bytes = std::fs::read(&original).unwrap();
    std::fs::write(dir.path().join("My song_2.lime"), b"keep this").unwrap();
    std::fs::create_dir(dir.path().join("My song_3.lime")).unwrap();
    doc.execute(Command::SetTempo(Tempo::from_bpm(101.0))).unwrap();
    let next = save_new_version(doc.project(), &original).unwrap();
    assert_eq!(next, dir.path().join("My song_4.lime"));
    assert_eq!(std::fs::read(&original).unwrap(), original_bytes);
    assert_eq!(std::fs::read(dir.path().join("My song_2.lime")).unwrap(), b"keep this");
    let loaded = load(&next).unwrap();
    assert_eq!(loaded.tempo, Tempo::from_bpm(101.0));
    let Instrument::Sampler(s) = &loaded.channels[0].instrument else {
        panic!("sampler")
    };
    assert_eq!(s.sample.as_ref().unwrap().path, "samples/kick.wav");
    assert_eq!(
        save_new_version(doc.project(), &next).unwrap(),
        dir.path().join("My song_5.lime")
    );
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        5,
        "no temporary files remain"
    );
}

#[test]
fn concurrent_versions_publish_distinct_complete_projects() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("並列.lime");
    let project = Project::new();
    save(&project, &original).unwrap();
    let original_bytes = std::fs::read(&original).unwrap();
    let barrier = std::sync::Barrier::new(8);
    let paths = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    save_new_version(&project, &original).unwrap()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|w| w.join().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
    });
    assert_eq!(paths.len(), 8);
    for path in paths {
        assert_eq!(load(&path).unwrap(), project);
    }
    assert_eq!(std::fs::read(&original).unwrap(), original_bytes);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 9);
}

#[test]
fn invalid_version_or_missing_directory_leaves_no_files() {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "missing/song.lime",
        "song_18446744073709551615.lime",
        "song_18446744073709551616.lime",
    ] {
        assert!(save_new_version(&Project::new(), &dir.path().join(name)).is_err());
    }
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}
