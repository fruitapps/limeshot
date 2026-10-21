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

/// A project saved by FL Studio 26.1.1 (see `fixtures/flp/fl26/README.md`).
#[test]
fn fl26_saved_steps_project() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp/fl26");
    let (p, r) = flp::import_file(&dir.join("lime-steps.flp"), None).expect("import");
    assert_eq!(r.version.as_deref().map(|v| v.starts_with("26.1")), Some(true));
    assert_eq!(p.tempo, Tempo::from_bpm(130.0), "tempo after the 3-byte event 172");
    let names: Vec<&str> = p.channels.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        ["808 Kick", "808 Clap", "808 HiHat", "808 Snare", "808 Astronomic"]
    );
    let inserts: Vec<u16> = p.channels.iter().map(|c| c.mixer_track).collect();
    assert_eq!(inserts, [1, 2, 3, 4, 5], "event 104");
    let steps = |i: usize| -> Vec<Tick> {
        p.patterns[0]
            .notes_for(p.channels[i].id)
            .map(|n| n.start / STEP_TICKS)
            .collect()
    };
    assert_eq!(steps(0), [0, 4, 8, 12]);
    assert_eq!(steps(1), [4, 12]);
    assert_eq!(steps(2), [0, 2, 4, 6, 8, 10, 12, 14]);
    assert!(
        p.patterns[0]
            .notes
            .iter()
            .all(|n| (n.velocity - 100.0 / 128.0).abs() < 1e-6)
    );
}

/// Piano-roll notes on a 3x Osc channel and a second, named pattern (FL Studio 26.1.1).
#[test]
fn fl26_saved_notes_project() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp/fl26");
    let (p, r) = flp::import_file(&dir.join("lime-notes.flp"), None).expect("import");
    assert_eq!(p.channels[5].name, "3x Osc");
    assert!(matches!(p.channels[5].instrument, Instrument::Synth(_)));
    assert!(r.replaced_generators.contains(&"3x Osc".to_string()));
    let osc = p.channels[5].id;
    let notes = |i: usize| -> Vec<(u8, Tick, Tick, u8)> {
        p.patterns[i]
            .notes
            .iter()
            .inspect(|n| assert_eq!(n.channel, osc))
            .map(|n| (n.key, n.start, n.length, (n.velocity * 128.0).round() as u8))
            .collect()
    };
    // FL shows key 60 as C5 and velocity as a percentage of 128 (34 % = 44).
    assert_eq!(
        notes(0),
        [
            (60, 0, 960, 44),
            (64, 960, 480, 100),
            (67, 1920, 1920, 125),
            (48, 3840, 3840, 100)
        ]
    );
    assert_eq!(
        p.patterns[1].name, "Bassline",
        "the name comes after the channel list, under a second event 65"
    );
    assert_eq!(notes(1), [(48, 0, 3840, 125), (46, 960, 3840, 125)]);
}

/// Playlist clips, an audio clip, an automation clip, a muted clip, 128 BPM and 3/4
/// (FL Studio 26.1.1, 88-byte playlist item records).
#[test]
fn fl26_saved_arrangement_project() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/flp/fl26");
    let (p, r) = flp::import_file(&dir.join("lime-arrange.flp"), Some(Path::new("/FL"))).expect("import");
    assert_eq!(
        (p.tempo, p.time_signature),
        (Tempo::from_bpm(128.0), TimeSignature::new(3, 4))
    );
    assert_eq!(
        (r.clips, r.skipped_automation_channels, r.skipped_items),
        (5, 1, 1),
        "the automation clip is reported"
    );
    let breaker = p
        .channels
        .iter()
        .find(|c| c.name == "DL Breaker")
        .expect("audio channel");
    match &breaker.instrument {
        Instrument::Sampler(s) => {
            assert_eq!(
                s.sample.as_ref().map(|s| s.path.as_str()),
                Some("/FL/Data/Patches/Packs/Loops/DL Breaker.wav")
            );
            assert!(!s.one_shot, "audio-clip channels play for the clip's length");
        }
        o => panic!("{o:?}"),
    }
    let (p1, bass) = (p.patterns[0].id, p.patterns[1].id);
    let clips: Vec<(ClipSource, u32, Tick, Tick, bool)> = p
        .playlist
        .clips
        .iter()
        .map(|c| (c.source, c.track, c.start, c.length, c.muted))
        .collect();
    // Positions were placed at 4/4 bars 1, 3, 5 and 9 (3840 LimeShot ticks per 4/4 bar); FL
    // stretched the pattern clips to whole 3/4 bars when the signature changed.
    assert_eq!(
        clips,
        [
            (ClipSource::Pattern(p1), 0, 0, 8640, false),
            (ClipSource::Audio(breaker.id), 4, 7680, 7680, false),
            (ClipSource::Pattern(bass), 1, 7680, 5760, false),
            (ClipSource::Pattern(p1), 0, 15360, 8640, false),
            (ClipSource::Pattern(bass), 2, 30720, 5760, true),
        ]
    );
}
