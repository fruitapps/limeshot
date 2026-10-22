//! Plugin channels: notes reach a channel-level instrument as events at exact frame offsets.

use limeshot_core::*;
use limeshot_dsp::{Instrument as PluginInstrument, NoteEvent, NoteEventKind, Transport};
use limeshot_engine::*;
use limeshot_sequencer::{NoSamples, compile};
use std::sync::{Arc, Mutex};

const SR: u32 = 48_000;

/// Outputs 1.0 on the left (and the held count on the right) while any key is held, and logs
/// every event with the engine frame it arrived at.
struct Gate {
    held: usize,
    frame: u64,
    log: Arc<Mutex<Vec<(u64, bool, u8)>>>,
}

impl PluginInstrument for Gate {
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], _t: &Transport) {
        let mut next = 0;
        for (i, f) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            while let Some(e) = notes.get(next).filter(|e| e.offset as usize == i) {
                let (on, key) = match e.kind {
                    NoteEventKind::On { key, .. } => (true, key),
                    NoteEventKind::Off { key, .. } => (false, key),
                };
                self.held = if on { self.held + 1 } else { self.held.saturating_sub(1) };
                if let Ok(mut l) = self.log.try_lock() {
                    l.push((self.frame + i as u64, on, key));
                }
                next += 1;
            }
            f[0] = if self.held > 0 { 1.0 } else { 0.0 };
            f[1] = self.held as f32;
        }
        assert_eq!(next, notes.len(), "every event offset is inside the block");
        self.frame += (out.len() / 2) as u64;
    }

    fn reset(&mut self) {}
}

fn project() -> (Document, ChannelId) {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let ch = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Plugin".into(),
            instrument: Instrument::Plugin(PluginRef::vst3("/x.vst3", "00", "Gate")),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::SetChannelVolume {
        channel: ch,
        volume: 1.0,
    })
    .unwrap();
    doc.execute(Command::SetMixerVolume { track: 1, volume: 1.0 }).unwrap();
    for (key, start, length) in [(60, 960, 480), (64, 1920, 960), (60, 2400, 240)] {
        doc.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: ch,
                key,
                start,
                length,
                velocity: 1.0,
            },
        })
        .unwrap();
    }
    (doc, ch)
}

#[test]
fn plugin_notes_start_and_stop_on_their_samples_at_any_block_size() {
    let (doc, ch) = project();
    let pat = doc.project().patterns[0].id;
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    // At 120 BPM and 48 kHz a quarter note (960 ticks) is 24 000 frames.
    let expected = vec![
        (24_000, true, 60),
        (36_000, false, 60),
        (48_000, true, 64),
        (60_000, true, 60),
        (66_000, false, 60),
        (72_000, false, 64),
    ];
    let mut reference: Option<Vec<f32>> = None;
    for block in [1, 64, 333, 1024] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let gate = Gate {
            held: 0,
            frame: 0,
            log: log.clone(),
        };
        let processors = Processors {
            effects: Vec::new(),
            instruments: vec![(ch.0, Box::new(gate))],
        };
        let settings = RenderSettings {
            sample_rate: SR,
            range: ExportRange::Pattern,
            max_tail_secs: 0.1,
            block,
        };
        let (audio, _) = render_offline_with(snap.clone(), &settings, processors).unwrap();
        assert_eq!(*log.lock().unwrap(), expected, "block {block}");
        let left: Vec<f32> = audio.chunks(2).map(|f| f[0]).collect();
        // Channel and mixer at unity, centred: the equal-power pan law scales both sides.
        let on = left[30_000];
        assert!(on > 0.5, "{on}");
        assert_eq!(left[23_999], 0.0);
        assert_eq!(left[24_000], on);
        assert_eq!(left[35_999], on);
        assert_eq!(left[36_000], 0.0);
        match &reference {
            Some(r) => assert_eq!(&audio, r, "block {block} changes the output"),
            None => reference = Some(audio),
        }
    }
}

#[test]
fn stopping_releases_plugin_notes_and_previews_play_without_transport() {
    let (doc, ch) = project();
    let pat = doc.project().patterns[0].id;
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut r = Renderer::new(SR, snap);
    r.adopt_instrument(
        ch.0,
        Box::new(Gate {
            held: 0,
            frame: 0,
            log: log.clone(),
        }),
    );
    r.rearrange();
    let mut buf = vec![0.0f32; 2 * 1000];
    r.play();
    for _ in 0..30 {
        r.render(&mut buf);
    }
    // 30 000 frames in: the first note (24 000–36 000) is held.
    assert!(buf[2 * 999] > 0.5);
    r.stop();
    r.render(&mut buf);
    assert_eq!(buf[0], 0.0, "stop releases the held note at once");
    assert_eq!(log.lock().unwrap().last(), Some(&(30_000, false, 60)));

    let ch_index = 0;
    r.preview_note_on(ch_index, 72, 0.8);
    r.render(&mut buf);
    assert!(buf[0] > 0.0, "an audition sounds while stopped");
    r.preview_note_off(ch_index, 72);
    r.render(&mut buf);
    assert_eq!(buf[0], 0.0);
    // Replacing the snapshot without the plugin channel retires the instrument.
    r.set_snapshot(Arc::new(limeshot_sequencer::Snapshot::default()));
    assert!(r.take_retired_instrument().is_some());
}

/// Owner's list item 8: Serum kept playing a note drawn in the Piano Roll. An audition whose
/// note-off arrived in the block its note-on started in sent the off first (both at offset 0,
/// note-offs sort first), so the plugin never released it. Every audition now ends.
#[test]
fn an_audition_released_in_the_block_it_started_still_ends() {
    let (doc, ch) = project();
    let pat = doc.project().patterns[0].id;
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut r = Renderer::new(SR, snap);
    r.adopt_instrument(
        ch.0,
        Box::new(Gate {
            held: 0,
            frame: 0,
            log: log.clone(),
        }),
    );
    r.rearrange();
    for block in [1usize, 64, 512] {
        let mut buf = vec![0.0f32; 2 * block];
        log.lock().unwrap().clear();
        // On and off before the next block, then a quick retrigger of another key.
        r.preview_note_on(0, 72, 0.8);
        r.preview_note_off(0, 72);
        r.preview_note_on(0, 74, 0.8);
        r.preview_note_on(0, 74, 0.8);
        r.preview_note_off(0, 74);
        for _ in 0..3 {
            r.render(&mut buf);
        }
        let l = log.lock().unwrap().clone();
        let ends = |key: u8| {
            let on = l.iter().position(|e| e.1 && e.2 == key).expect("on");
            let off = l.iter().rposition(|e| !e.1 && e.2 == key).expect("off");
            assert!(off > on, "block {block}: the off of {key} after its on: {l:?}");
        };
        ends(72);
        ends(74);
        assert_eq!(buf[2 * block - 1], 0.0, "block {block}: nothing is held: {l:?}");
    }
}

/// Owner's list item 10: Stop while stopped ends held plugin auditions too.
#[test]
fn stop_while_stopped_ends_plugin_auditions() {
    let (doc, ch) = project();
    let pat = doc.project().patterns[0].id;
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut r = Renderer::new(SR, Arc::new(compile(doc.project(), pat, &NoSamples)));
    r.adopt_instrument(
        ch.0,
        Box::new(Gate {
            held: 0,
            frame: 0,
            log: log.clone(),
        }),
    );
    r.rearrange();
    let mut buf = vec![0.0f32; 2 * 256];
    r.preview_note_on(0, 72, 0.8);
    r.render(&mut buf);
    assert!(buf[0] > 0.0);
    r.stop();
    r.render(&mut buf);
    assert_eq!(buf[2 * 255], 0.0, "the audition ended");
    assert_eq!(log.lock().unwrap().last().map(|e| (e.1, e.2)), Some((false, 72)));
}

/// Logs every automation point a plugin gets, with the engine frame it lands on.
struct ParamLog {
    frame: u64,
    log: Arc<Mutex<Vec<(u64, u32, f64)>>>,
    pending: Vec<(u32, u32, f64)>,
}

impl ParamLog {
    fn new(log: Arc<Mutex<Vec<(u64, u32, f64)>>>) -> Self {
        Self {
            frame: 0,
            log,
            pending: Vec::with_capacity(4096),
        }
    }

    fn take(&mut self, frames: usize) {
        if let Ok(mut l) = self.log.try_lock() {
            for &(id, offset, v) in &self.pending {
                assert!((offset as usize) < frames.max(1), "offset inside the block");
                l.push((self.frame + u64::from(offset), id, v));
            }
        }
        self.pending.clear();
        self.frame += frames as u64;
    }
}

impl PluginInstrument for ParamLog {
    fn process(&mut self, out: &mut [f32], _notes: &[NoteEvent], _t: &Transport) {
        out.fill(0.0);
        self.take(out.len() / 2);
    }
    fn reset(&mut self) {}
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.pending.push((id, offset, value));
    }
}

impl limeshot_dsp::effects::Effect for ParamLog {
    fn set_params(&mut self, _params: &[f32]) {}
    fn process(&mut self, buf: &mut [f32]) {
        self.take(buf.len() / 2);
    }
    fn reset(&mut self) {}
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.pending.push((id, offset, value));
    }
}

#[test]
fn plugin_parameter_automation_reaches_channel_and_insert_plugins_at_its_frames() {
    let (mut doc, ch) = project();
    let effect = doc
        .execute(Command::AddPluginEffect {
            track: 1,
            plugin: PluginRef::vst3("/fx.vst3", "01", "Fx"),
        })
        .unwrap()
        .0
        .unwrap();
    // A ramp from 0 to 1 over one bar (4 beats at 120 BPM = 96 000 frames) on the channel's
    // plugin parameter 7, and a falling one on the insert's parameter 3.
    for (plugin, id, from, to) in [
        (PluginTarget::Channel(ch), 7, 0.0, 1.0),
        (PluginTarget::Effect { track: 1, effect }, 3, 1.0, 0.0),
    ] {
        let clip = ClipId(
            doc.execute(Command::AddAutomationClip {
                target: AutomationTarget::PluginParam { plugin, id },
                track: if id == 7 { 0 } else { 1 },
                start: 0,
                length: 3840,
                value: Some(from),
                name: Some("Ramp".into()),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        let a = doc.project().clip(clip).unwrap().automation().unwrap();
        assert_eq!(doc.project().automation(a).unwrap().name, "Ramp");
        doc.execute(Command::SetAutomationPoints {
            automation: a,
            points: vec![
                AutomationPoint {
                    tick: 0,
                    value: from,
                    tension: 0.0,
                },
                AutomationPoint {
                    tick: 3840,
                    value: to,
                    tension: 0.0,
                },
            ],
        })
        .unwrap();
    }
    let pat = doc.project().patterns[0].id;
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    for block in [64, 333, 1024] {
        let (inst_log, fx_log) = (Arc::new(Mutex::new(Vec::new())), Arc::new(Mutex::new(Vec::new())));
        let processors = Processors {
            effects: vec![(effect, Box::new(ParamLog::new(fx_log.clone())))],
            instruments: vec![(ch.0, Box::new(ParamLog::new(inst_log.clone())))],
        };
        let settings = RenderSettings {
            sample_rate: SR,
            range: ExportRange::Song,
            max_tail_secs: 0.0,
            block,
        };
        render_offline_with(snap.clone(), &settings, processors).unwrap();
        for (log, id, rising) in [(inst_log, 7, true), (fx_log, 3, false)] {
            let log = log.lock().unwrap();
            assert!(log.len() > 1000, "points every 64 frames or less: {}", log.len());
            let mut last = 0;
            for &(frame, pid, v) in log.iter().filter(|p| p.0 < 96_000) {
                assert_eq!(pid, id);
                assert!(frame >= last, "in order");
                assert!(frame - last <= 64 || last == 0, "a point at least every 64 frames");
                last = frame;
                let want = frame as f64 / 96_000.0;
                let want = if rising { want } else { 1.0 - want };
                assert!((v - want).abs() < 1e-3, "block {block}, frame {frame}: {v} vs {want}");
            }
        }
    }
}
