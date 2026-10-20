//! `.flp` import against the fixture files (and FL Studio's own projects when installed).

use limeshot_core::*;
use limeshot_project::flp;
use std::path::Path;

fn import(name: &str) -> (Project, flp::ImportReport) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp");
    flp::import_file(&dir.join(name), Some(Path::new("/FL"))).expect(name)
}

#[test]
fn fixture_files_match_their_definitions() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp");
    for (name, bytes) in flp::fixtures() {
        assert_eq!(
            std::fs::read(dir.join(name)).expect(name),
            bytes,
            "{name}: run make_flp_fixtures"
        );
    }
}

#[test]
fn steps_become_step_notes_with_samples_levels_and_clips() {
    let (p, r) = import("steps.flp");
    assert_eq!((p.title.as_str(), p.tempo), ("Steps", Tempo::from_bpm(120.0)));
    assert_eq!((r.channels, r.patterns, r.clips), (3, 1, 2));
    let kick = &p.channels[0];
    match &kick.instrument {
        Instrument::Sampler(s) => assert_eq!(s.sample.as_ref().map(|s| s.path.as_str()), Some("/FL/Data/Kick.wav")),
        o => panic!("{o:?}"),
    }
    assert_eq!(
        (kick.mixer_track, p.channels[1].volume, p.channels[1].pan),
        (1, 1.0, -0.5)
    );
    let pat = &p.patterns[0];
    assert_eq!(pat.name, "Beat");
    assert!(
        pat.is_step_sequence(kick.id, 60),
        "zero-length FL steps are LimeShot steps"
    );
    assert_eq!(
        pat.notes_for(kick.id).map(|n| n.start).collect::<Vec<_>>(),
        vec![0, 960, 1920, 2880]
    );
    assert_eq!(p.playlist.clips[1].start, 3840);
}

#[test]
fn melody_keeps_lengths_velocity_signature_and_offsets() {
    let (p, r) = import("melody.flp");
    assert_eq!(
        (p.tempo, p.time_signature),
        (Tempo::from_bpm(128.5), TimeSignature::new(3, 4))
    );
    assert_eq!(r.replaced_generators, vec!["Lead".to_string()]);
    let n: Vec<(u8, Tick, Tick, f32)> = p.patterns[0]
        .notes
        .iter()
        .map(|n| (n.key, n.start, n.length, n.velocity))
        .collect();
    assert_eq!(
        n,
        vec![
            (60, 0, 960, 100.0 / 128.0),
            (64, 960, 480, 0.625),
            (67, 1440, 1440, 1.0)
        ]
    );
    assert_eq!(p.patterns[1].name, "Pattern 2");
    let c = &p.playlist.clips;
    assert_eq!((c[1].start, c[1].length, c[1].offset, c[2].track), (2880, 1440, 960, 3));
}

#[test]
fn audio_clips_import_and_automation_is_reported() {
    let (p, r) = import("audio_and_automation.flp");
    assert_eq!(
        (r.channels, r.clips, r.skipped_automation_channels, r.skipped_items),
        (1, 1, 1, 1)
    );
    let c = &p.playlist.clips[0];
    assert_eq!((c.start, c.length, c.track), (1920, 7680, 6));
    assert!(matches!(c.source, ClipSource::Audio(_)));
}

#[test]
fn other_time_bases_old_records_and_truncation() {
    let (p, _) = import("ppq192.flp");
    assert_eq!(p.patterns[0].notes[0].length, 3840, "768 ticks at 192 PPQ = one bar");
    assert_eq!((p.playlist.clips[0].start, p.playlist.clips[0].track), (3840, 1));
    let (p, r) = import("truncated.flp");
    assert_eq!((r.channels, r.clips), (3, 0), "complete events are kept");
    assert!(!p.patterns[0].notes.is_empty());
    let (_, r) = import("empty.flp");
    assert_eq!((r.channels, r.notes), (1, 0));
}

/// FL Studio's own templates and demos, when FL is installed (skipped otherwise).
#[test]
fn installed_fl_projects_all_import() {
    let Some(dir) = flp::default_factory_dir() else { return };
    let mut n = 0;
    let mut stack = vec![dir.join("Data")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "flp") {
                flp::import_file(&p, Some(&dir)).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                n += 1;
            }
        }
    }
    assert!(n > 0);
}
