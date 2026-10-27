use limeshot_core::*;

#[test]
fn arrangements_clone_switch_merge_delete_and_roundtrip() {
    let mut d = Document::default();
    let pattern = d.project().patterns[0].id;
    d.execute(Command::AddClip {
        pattern,
        track: 2,
        start: PPQ,
        length: Some(PPQ),
    })
    .unwrap();
    d.execute(Command::AddMarker {
        tick: PPQ,
        name: "Intro".into(),
    })
    .unwrap();
    let original = d.project().playlist.clone();
    d.execute(Command::CloneArrangement {
        index: 0,
        name: Some("Alternative".into()),
    })
    .unwrap();
    assert_eq!(d.project().current_arrangement, 1);
    assert_eq!(d.project().arrangement(0), Some(&original));
    assert_ne!(d.project().playlist.clips[0].id, original.clips[0].id);
    assert_ne!(d.project().playlist.markers[0].id, original.markers[0].id);
    assert_eq!(d.project().playlist.clips[0].source, ClipSource::Pattern(pattern));
    d.execute(Command::AddArrangement { name: "Empty".into() }).unwrap();
    assert_eq!(d.project().current_arrangement, 2);
    assert!(d.project().playlist.clips.is_empty());
    d.execute(Command::CloneArrangement {
        index: 0,
        name: Some("Inserted".into()),
    })
    .unwrap();
    assert_eq!(d.project().current_arrangement, 1);
    assert_eq!(d.project().arrangement(2).unwrap().name, "Alternative");
    assert_eq!(d.project().arrangement(3).unwrap().name, "Empty");
    d.execute(Command::SelectArrangement { index: 3 }).unwrap();
    d.execute(Command::MergeArrangement { source: 0 }).unwrap();
    assert_eq!(d.project().playlist.clips.len(), 1);
    assert_eq!(d.project().arrangement(0), Some(&original));
    let merged = d.project().clone();
    let json = serde_json::to_vec(&merged).unwrap();
    let mut reopened: Project = serde_json::from_slice(&json).unwrap();
    reopened.sanitize();
    assert_eq!(reopened, merged);
    d.execute(Command::DeleteArrangement { index: 0 }).unwrap();
    assert_eq!(d.project().current_arrangement, 2);
    assert_eq!(d.project().playlist.name, "Empty");
    d.undo();
    assert_eq!(d.project(), &merged);
    d.execute(Command::DeleteArrangement { index: 3 }).unwrap();
    assert_eq!(d.project().playlist.name, "Alternative");
    d.undo();
    assert_eq!(d.project(), &merged);
    d.execute(Command::RemovePattern(pattern)).unwrap_err(); // Last pattern cannot be removed.
    d.execute(Command::AddPattern { name: "Spare".into() }).unwrap();
    d.execute(Command::RemovePattern(pattern)).unwrap();
    assert!((0..d.project().arrangement_count()).all(|i| d.project().arrangement(i).unwrap().clips.is_empty()));
}

#[test]
fn grouped_tracks_and_custom_heights_survive_save_and_undo() {
    let mut d = Document::default();
    d.execute(Command::SetPlaylistTracksGrouped {
        tracks: vec![1, 3],
        grouped: true,
    })
    .unwrap();
    assert!(!d.project().playlist.tracks[1].grouped_with_above);
    assert!(d.project().playlist.tracks[2..=3].iter().all(|t| t.grouped_with_above));
    d.execute(Command::SetPlaylistTrackGroupCollapsed {
        track: 1,
        collapsed: true,
    })
    .unwrap();
    d.execute(Command::SetPlaylistTrackHeight {
        track: 1,
        height: Some(123.0),
        collapsed: false,
    })
    .unwrap();
    let before = d.project().clone();
    d.execute(Command::SetPlaylistTrackHeight {
        track: 1,
        height: None,
        collapsed: true,
    })
    .unwrap();
    d.undo();
    assert_eq!(d.project(), &before);
    d.execute(Command::SetPlaylistTrackSizeLock { track: 1, locked: true })
        .unwrap();
    d.execute(Command::SetPlaylistTrackHeight {
        track: 1,
        height: Some(400.0),
        collapsed: true,
    })
    .unwrap();
    assert_eq!(d.project().playlist.tracks[1].height, Some(123.0));
    assert!(!d.project().playlist.tracks[1].collapsed);
    let json = serde_json::to_vec(d.project()).unwrap();
    let p: Project = serde_json::from_slice(&json).unwrap();
    assert_eq!(&p, d.project());
    d.execute(Command::SetPlaylistTracksGrouped {
        tracks: vec![2, 3],
        grouped: false,
    })
    .unwrap();
    assert!(!d.project().playlist.tracks[2].grouped_with_above);
}
