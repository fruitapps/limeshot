use limeshot_core::*;

/// Build linked clips of every source kind with fades and unknown metadata for clipboard checks.
fn fixture() -> Project {
    let mut doc = Document::default();
    doc.execute(Command::AddChannel {
        name: "Audio".into(),
        instrument: Instrument::Sampler(SamplerParams::default()),
    })
    .unwrap();
    let channel = doc.project().channels[0].id;
    doc.execute(Command::AddAutomationClip {
        target: AutomationTarget::Tempo,
        track: 2,
        start: PPQ * 3,
        length: PPQ * 2,
        value: None,
        name: None,
    })
    .unwrap();
    let mut p = doc.project().clone();
    let sources = [
        ClipSource::Pattern(p.patterns[0].id),
        ClipSource::Audio(channel),
        p.playlist.clips[0].source,
    ];
    p.playlist.clips.clear();
    for (i, source) in sources.into_iter().enumerate() {
        let id = ClipId(p.alloc_id());
        p.playlist.clips.push(Clip {
            id,
            source,
            track: i as u32,
            start: (i as u64 + 1) * PPQ,
            length: PPQ * 2,
            offset: 120,
            muted: true,
            fade_in: 90,
            fade_out: 180,
            extra: serde_json::from_value(serde_json::json!({
                "fade-in-curve": 0.5,
                "fade-out-curve": -0.25,
                "length-manual": true,
                "future-clip-data": { "value": 42 }
            }))
            .unwrap(),
        });
    }
    p
}

/// Pasting round-tripped commands preserves properties and timing across undo, redo and serialization.
#[test]
fn paste_keeps_all_clip_kinds_properties_relative_timing_and_fresh_ids() {
    let p = fixture();
    let mut clips = p.playlist.clips.clone();
    clips.reverse(); // Origin is the earliest time, independent of selection/storage order.
    let mut doc = Document::new(p.clone());
    let cmd = Command::PasteClips {
        clips: clips.clone(),
        start: PPQ * 8,
    };
    // CLI/script command round trip retains the complete copied snapshots.
    let cmd = serde_json::from_str(&serde_json::to_string(&cmd).unwrap()).unwrap();
    doc.execute(cmd).unwrap();
    let pasted = &doc.project().playlist.clips[3..];
    for (original, pasted) in clips.iter().zip(pasted) {
        assert!(!clips.iter().any(|c| c.id == pasted.id));
        let mut expected = original.clone();
        expected.id = pasted.id;
        expected.start += PPQ * 7;
        assert_eq!(pasted, &expected);
    }
    let after = doc.project().clone();
    doc.undo();
    assert_eq!(doc.project(), &p);
    assert!(!doc.is_modified());
    doc.redo();
    assert_eq!(doc.project(), &after);
    let reopened: Project = serde_json::from_str(&serde_json::to_string(doc.project()).unwrap()).unwrap();
    assert_eq!(reopened, after);
}

/// Invalid sources or tracks reject the entire paste without consuming IDs or changing history.
#[test]
fn invalid_paste_leaves_document_ids_history_and_saved_state_unchanged() {
    for source in [
        ClipSource::Pattern(PatternId(u32::MAX)),
        ClipSource::Audio(ChannelId(u32::MAX)),
        ClipSource::Automation(AutomationId(u32::MAX)),
    ] {
        let p = fixture();
        let mut clips = p.playlist.clips.clone();
        clips[1].source = source;
        let mut doc = Document::new(p.clone());
        assert!(doc.execute(Command::PasteClips { clips, start: 0 }).is_err());
        assert_eq!(doc.project(), &p);
        assert_eq!(doc.revision(), 0);
        assert!(!doc.is_modified());
        assert!(doc.undo_label().is_none());
    }
    let p = fixture();
    let mut clips = p.playlist.clips.clone();
    clips[2].track = u32::MAX;
    let mut doc = Document::new(p.clone());
    assert!(doc.execute(Command::PasteClips { clips, start: 0 }).is_err());
    assert_eq!(doc.project(), &p);
}

/// Empty pastes do not edit the document, and extreme destination ticks saturate safely.
#[test]
fn empty_paste_is_a_noop_and_extreme_destination_does_not_overflow() {
    let mut doc = Document::new(fixture());
    doc.execute(Command::PasteClips {
        clips: vec![],
        start: 0,
    })
    .unwrap();
    assert_eq!(doc.revision(), 0);
    let clips = doc.project().playlist.clips.clone();
    doc.execute(Command::PasteClips { clips, start: u64::MAX }).unwrap();
    assert!(doc.project().playlist.clips[3..].iter().all(|c| c.start == u64::MAX));
}

/// Hostile fade lengths are bounded after placement normalizes zero-length clips, including undo/redo.
#[test]
fn paste_clamps_fades_to_final_length_and_undoes_the_normalized_snapshot() {
    for (length, fade_in, fade_out, expected) in [
        (0, u64::MAX, u64::MAX, (1, 0)),
        (0, 0, u64::MAX, (0, 1)),
        (PPQ, u64::MAX, u64::MAX, (PPQ, 0)),
        (PPQ, PPQ / 4, u64::MAX, (PPQ / 4, 3 * PPQ / 4)),
    ] {
        let before = fixture();
        let mut clip = before.playlist.clips[1].clone();
        clip.length = length;
        clip.fade_in = fade_in;
        clip.fade_out = fade_out;
        let mut doc = Document::new(before.clone());
        doc.execute(Command::PasteClips {
            clips: vec![clip],
            start: 0,
        })
        .unwrap();
        let pasted = doc.project().playlist.clips.last().unwrap();
        assert_eq!(pasted.length, length.max(1));
        assert_eq!((pasted.fade_in, pasted.fade_out), expected);
        assert!(pasted.fade_in + pasted.fade_out <= pasted.length);
        let after = doc.project().clone();
        assert!(doc.undo());
        assert_eq!(doc.project(), &before);
        assert!(doc.redo());
        assert_eq!(doc.project(), &after);
    }
}

/// Restoring saved manual fades after resizing and pasting at tick zero keeps handle arithmetic bounded.
#[test]
fn shortened_automatic_clip_pasted_at_zero_clamps_restored_manual_fades() {
    let mut p = fixture();
    let mut audio = p.playlist.clips[1].clone();
    audio.muted = false;
    audio.start = 0;
    audio.track = 0;
    audio.fade_in = PPQ / 4;
    audio.fade_out = PPQ * 3 / 2;
    let mut second = audio.clone();
    second.id = ClipId(p.alloc_id());
    second.start = PPQ;
    p.playlist.clips = vec![audio.clone(), second];
    let mut doc = Document::new(p);
    doc.execute(Command::SetAutoCrossfades(true)).unwrap();
    doc.execute(Command::ResizeClip {
        clip: audio.id,
        length: PPQ * 5 / 4,
    })
    .unwrap();
    let mut copied = doc.project().clip(audio.id).unwrap().clone();
    assert!(copied.extra.contains_key("auto-crossfade-base-v1"));
    assert!(audio.fade_out > copied.length);
    // Keep the destination isolated so its restored manual fades remain visible.
    copied.track = 1;
    let before = doc.project().clone();
    doc.execute(Command::PasteClips {
        clips: vec![copied],
        start: 0,
    })
    .unwrap();
    let pasted = doc.project().playlist.clips.last().unwrap();
    assert_eq!(pasted.start, 0);
    assert_eq!((pasted.fade_in, pasted.fade_out), (PPQ / 4, PPQ));
    assert_eq!(pasted.end() - pasted.fade_out, PPQ / 4);
    assert!(!pasted.extra.contains_key("auto-crossfade-base-v1"));
    let after = doc.project().clone();
    assert!(doc.undo());
    assert_eq!(doc.project(), &before);
    assert!(doc.redo());
    assert_eq!(doc.project(), &after);
}

/// Copied automatic fades restore their manual bases before destination overlap recalculation.
#[test]
fn paste_restores_manual_fades_then_recalculates_automatic_destination_overlaps() {
    let mut p = fixture();
    let mut audio = p.playlist.clips[1].clone();
    audio.muted = false;
    audio.start = 0;
    audio.track = 0;
    let mut second = audio.clone();
    second.id = ClipId(p.alloc_id());
    second.start = PPQ;
    p.playlist.clips = vec![audio.clone(), second];
    let mut doc = Document::new(p);
    doc.execute(Command::SetAutoCrossfades(true)).unwrap();
    let copied = doc.project().playlist.clips.clone();
    assert_eq!(copied[0].fade_out, PPQ);
    doc.execute(Command::PasteClips {
        clips: vec![copied[0].clone()],
        start: PPQ * 8,
    })
    .unwrap();
    let isolated = doc.project().playlist.clips.last().unwrap();
    assert_eq!((isolated.fade_in, isolated.fade_out), (audio.fade_in, audio.fade_out));
    assert!(!isolated.extra.contains_key("auto-crossfade-base-v1"));
    doc.execute(Command::PasteClips {
        clips: copied,
        start: PPQ * 16,
    })
    .unwrap();
    let pasted = &doc.project().playlist.clips[3..];
    assert_eq!(pasted[0].fade_out, PPQ);
    assert_eq!(pasted[1].fade_in, PPQ);
    doc.execute(Command::SetAutoCrossfades(false)).unwrap();
    for clip in &doc.project().playlist.clips {
        assert_eq!((clip.fade_in, clip.fade_out), (audio.fade_in, audio.fade_out));
    }
}
