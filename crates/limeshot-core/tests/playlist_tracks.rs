use limeshot_core::*;

#[test]
fn playlist_track_edits_keep_clips_attached_and_undo_as_one_action() {
    let mut doc = Document::default();
    let pattern = doc.project().patterns[0].id;
    let id = ClipId(
        doc.execute(Command::AddClip {
            pattern,
            track: 1,
            start: PPQ,
            length: Some(PPQ),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::RenamePlaylistTrack {
        track: 1,
        name: "Lead".into(),
    })
    .unwrap();
    doc.execute(Command::SetPlaylistTrackMuteLock { track: 1, locked: true })
        .unwrap();
    let original = doc.project().clone();
    doc.execute(Command::InsertPlaylistTrack { track: 0 }).unwrap();
    assert_eq!(doc.project().clip(id).unwrap().track, 2);
    assert_eq!(doc.project().playlist.tracks[2].name, "Lead");
    doc.execute(Command::MovePlaylistTrack { track: 2, delta: -2 }).unwrap();
    assert_eq!(doc.project().clip(id).unwrap().track, 0);
    assert_eq!(doc.project().playlist.tracks[0].name, "Lead");
    assert!(doc.project().playlist.tracks[0].mute_locked);
    doc.execute(Command::RemovePlaylistTrack { track: 0 }).unwrap();
    assert!(doc.project().clip(id).is_none());
    doc.undo();
    assert_eq!(doc.project().clip(id).unwrap().track, 0);
    doc.undo();
    doc.undo();
    assert_eq!(doc.project(), &original);
    let saved = serde_json::to_string(doc.project()).unwrap();
    let reopened: Project = serde_json::from_str(&saved).unwrap();
    assert!(reopened.playlist.tracks[1].mute_locked);
}

#[test]
fn track_clone_preserves_mutes_offsets_and_linked_sources() {
    let mut doc = Document::default();
    let pattern = doc.project().patterns[0].id;
    let id = ClipId(
        doc.execute(Command::PlaceClip {
            source: ClipSource::Pattern(pattern),
            track: 0,
            start: PPQ,
            length: PPQ * 2,
            offset: 120,
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::SetClipMuted { clip: id, muted: true }).unwrap();
    doc.execute(Command::SetPlaylistTrackMute { track: 0, muted: true })
        .unwrap();
    doc.execute(Command::SetPlaylistTrackMuteLock { track: 0, locked: true })
        .unwrap();
    let original = doc.project().clone();
    doc.execute(Command::ClonePlaylistTrack(0)).unwrap();
    let cloned = doc.project().playlist.clips.last().unwrap();
    assert_ne!(cloned.id, id);
    assert_eq!(
        (cloned.source, cloned.track, cloned.offset, cloned.muted),
        (ClipSource::Pattern(pattern), 1, 120, true)
    );
    assert!(doc.project().playlist.tracks[1].muted);
    assert!(doc.project().playlist.tracks[1].mute_locked);
    doc.undo();
    assert_eq!(doc.project(), &original);
}
