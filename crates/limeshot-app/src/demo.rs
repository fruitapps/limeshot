//! A short demo song built only through commands, the same way the UI builds projects. It doubles
//! as the scripted form of the MVP workflow (drums from samples, a synth melody, several
//! patterns arranged on the Playlist, mixer routing, volume and pan).

use limeshot_core::*;
use std::path::Path;

fn id(doc: &mut Document, cmd: Command) -> Result<u32, String> {
    doc.execute(cmd).map_err(|e| e.to_string())?.0.ok_or_else(|| "command created nothing".to_string())
}

fn run(doc: &mut Document, cmd: Command) -> Result<(), String> {
    doc.execute(cmd).map(|_| ()).map_err(|e| e.to_string())
}

fn sampler(doc: &mut Document, name: &str, file: &Path) -> Result<ChannelId, String> {
    let p = SamplerParams { sample: Some(SampleRef { path: file.to_string_lossy().into_owned() }), ..Default::default() };
    Ok(ChannelId(id(doc, Command::AddChannel { name: name.into(), instrument: Instrument::Sampler(p) })?))
}

/// Build the demo. `samples` is the folder containing the bundled `Lime *.wav` files.
pub fn build(samples: &Path) -> Result<Document, String> {
    let mut d = Document::default();
    run(&mut d, Command::SetTitle("Lime Demo".into()))?;
    run(&mut d, Command::SetTempo(Tempo::from_bpm(124.0)))?;
    let beat = d.project().patterns.first().map(|p| p.id).ok_or("no pattern")?;
    run(&mut d, Command::RenamePattern { pattern: beat, name: "Beat".into() })?;

    let kick = sampler(&mut d, "Kick", &samples.join("Lime Kick.wav"))?;
    let clap = sampler(&mut d, "Clap", &samples.join("Lime Clap.wav"))?;
    let hat = sampler(&mut d, "Hat", &samples.join("Lime Hat Closed.wav"))?;
    for s in [0, 4, 8, 12] {
        run(&mut d, Command::SetStep { pattern: beat, channel: kick, step: s, on: true })?;
    }
    for s in [4, 12] {
        run(&mut d, Command::SetStep { pattern: beat, channel: clap, step: s, on: true })?;
    }
    for s in [2, 6, 10, 14, 15] {
        run(&mut d, Command::SetStep { pattern: beat, channel: hat, step: s, on: true })?;
    }

    let lead = ChannelId(id(
        &mut d,
        Command::AddChannel {
            name: "Lead".into(),
            instrument: Instrument::Synth(SynthParams { waveform: Waveform::Saw, detune_cents: 9.0, release_ms: 220.0, ..Default::default() }),
        },
    )?);
    let bass = ChannelId(id(
        &mut d,
        Command::AddChannel {
            name: "Bass".into(),
            instrument: Instrument::Synth(SynthParams {
                waveform: Waveform::Square,
                octave: -1,
                attack_ms: 2.0,
                decay_ms: 300.0,
                sustain: 0.5,
                release_ms: 80.0,
                ..Default::default()
            }),
        },
    )?);

    let melody = PatternId(id(&mut d, Command::AddPattern { name: "Melody".into() })?);
    // A two-bar phrase in A minor; (key, start in 16ths, length in 16ths).
    let phrase: [(u8, u64, u64); 10] =
        [(69, 0, 3), (72, 3, 3), (76, 6, 2), (74, 8, 4), (72, 12, 4), (69, 16, 3), (67, 19, 3), (64, 22, 2), (67, 24, 4), (69, 28, 4)];
    let notes = phrase
        .iter()
        .map(|&(key, s, l)| NewNote { channel: lead, key, start: s * STEP_TICKS, length: l * STEP_TICKS, velocity: 0.75 })
        .collect();
    run(&mut d, Command::AddNotes { pattern: melody, notes })?;

    let bassline = PatternId(id(&mut d, Command::AddPattern { name: "Bassline".into() })?);
    let roots: [u8; 4] = [45, 41, 43, 40]; // A, F, G, E
    let notes = roots
        .iter()
        .enumerate()
        .flat_map(|(bar, &k)| {
            [0u64, 6, 10].into_iter().map(move |s| NewNote {
                channel: bass,
                key: k,
                start: (bar as u64 * 16 + s) * STEP_TICKS,
                length: 3 * STEP_TICKS,
                velocity: 0.85,
            })
        })
        .collect();
    run(&mut d, Command::AddNotes { pattern: bassline, notes })?;

    let bar = d.project().time_signature.ticks_per_bar();
    for b in 0..8 {
        run(&mut d, Command::AddClip { pattern: beat, track: 0, start: b * bar, length: None })?;
    }
    for b in [2u64, 4, 6] {
        run(&mut d, Command::AddClip { pattern: melody, track: 1, start: b * bar, length: None })?;
    }
    run(&mut d, Command::AddClip { pattern: bassline, track: 2, start: 4 * bar, length: None })?;

    // Mix: channels already sit on inserts 1–5; set levels and pans.
    run(&mut d, Command::SetChannelPan { channel: hat, pan: 0.3 })?;
    run(&mut d, Command::SetChannelPan { channel: lead, pan: -0.15 })?;
    for (t, v) in [(1u16, 0.9f32), (2, 0.7), (3, 0.45), (4, 0.55), (5, 0.7)] {
        run(&mut d, Command::SetMixerVolume { track: t, volume: v })?;
    }
    for (t, name) in [(1u16, "Kick"), (2, "Clap"), (3, "Hat"), (4, "Lead"), (5, "Bass")] {
        run(&mut d, Command::RenameMixerTrack { track: t, name: name.into() })?;
    }
    run(&mut d, Command::RenamePlaylistTrack { track: 0, name: "Drums".into() })?;
    run(&mut d, Command::RenamePlaylistTrack { track: 1, name: "Melody".into() })?;
    run(&mut d, Command::RenamePlaylistTrack { track: 2, name: "Bass".into() })?;
    Ok(d)
}
