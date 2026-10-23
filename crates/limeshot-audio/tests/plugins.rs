//! Plugin instruments sent to the audio thread play in realtime.

use limeshot_audio::{Msg, Player};
use limeshot_core::*;
use limeshot_dsp::{Instrument as PluginInstrument, NoteEvent, NoteEventKind, Transport};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Full scale while any key is held.
struct Gate(usize);

impl PluginInstrument for Gate {
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], _t: &Transport) {
        let mut next = 0;
        for (i, f) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            while let Some(e) = notes.get(next).filter(|e| e.offset as usize == i) {
                match e.kind {
                    limeshot_dsp::NoteEventKind::ChannelPan { .. } => {}
                    NoteEventKind::On { .. } => self.0 += 1,
                    NoteEventKind::Off { .. } => self.0 = self.0.saturating_sub(1),
                    NoteEventKind::Clip { .. } => self.0 += 1,
                    NoteEventKind::Pan { .. } => {}
                }
                next += 1;
            }
            let v = if self.0 > 0 { 0.5 } else { 0.0 };
            f[0] = v;
            f[1] = v;
        }
    }
    fn reset(&mut self) {}
}

#[test]
fn a_plugin_channel_sounds_through_the_player() {
    let mut doc = Document::default();
    let pat = doc.project().patterns[0].id;
    let ch = doc
        .execute(Command::AddChannel {
            name: "P".into(),
            instrument: Instrument::Plugin(PluginRef::vst3("/x.vst3", "00", "Gate")),
        })
        .unwrap()
        .0
        .unwrap();
    doc.execute(Command::AddNote {
        pattern: pat,
        note: NewNote {
            channel: ChannelId(ch),
            key: 60,
            start: 0,
            length: 3840,
            velocity: 1.0,
        },
    })
    .unwrap();
    let snap = Arc::new(limeshot_sequencer::compile(
        doc.project(),
        pat,
        &limeshot_sequencer::NoSamples,
    ));
    let mut player = Player::silent("test".into(), snap.clone());
    assert!(player.send(Msg::Instrument(ch, Box::new(Gate(0)))));
    assert!(player.send(Msg::Snapshot(snap)));
    assert!(player.send(Msg::Play));
    let start = Instant::now();
    let mut peak = 0.0f32;
    while start.elapsed() < Duration::from_millis(500) && peak == 0.0 {
        std::thread::sleep(Duration::from_millis(10));
        let (l, r) = player.status().peak(0);
        peak = peak.max(l).max(r);
        player.collect_garbage();
    }
    assert!(peak > 0.1, "master peak {peak}");
}
