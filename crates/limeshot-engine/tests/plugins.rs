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
                    limeshot_dsp::NoteEventKind::ChannelPan { .. } => {
                        next += 1;
                        continue;
                    }
                    NoteEventKind::On { key, .. } => (true, key),
                    NoteEventKind::Off { key, .. } => (false, key),
                    NoteEventKind::Clip { key, .. } => (true, key),
                    NoteEventKind::Pan { .. } => {
                        next += 1;
                        continue;
                    }
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
            dither: false,
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
                    mode: Default::default(),
                },
                AutomationPoint {
                    tick: 3840,
                    value: to,
                    tension: 0.0,
                    mode: Default::default(),
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
            dither: false,
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

/// A plugin instrument that sounds an impulse `latency` frames after each note-on, and reports
/// that latency, as a look-ahead plugin would.
struct Click {
    latency: u32,
    /// Frames until the next impulse, if one is due.
    due: Vec<u64>,
    frame: u64,
}

impl PluginInstrument for Click {
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], _t: &Transport) {
        out.fill(0.0);
        for e in notes {
            if matches!(e.kind, NoteEventKind::On { .. }) {
                self.due
                    .push(self.frame + u64::from(e.offset) + u64::from(self.latency));
            }
        }
        let n = (out.len() / 2) as u64;
        for &d in &self.due {
            if d >= self.frame && d < self.frame + n {
                let i = (d - self.frame) as usize;
                out[2 * i] = 1.0;
                out[2 * i + 1] = 1.0;
            }
        }
        self.due.retain(|&d| d >= self.frame + n);
        self.frame += n;
    }
    fn reset(&mut self) {}
    fn latency(&self) -> u32 {
        self.latency
    }
}

/// An insert that delays its input by `latency` frames and reports it.
struct Lag {
    line: Vec<f32>,
    pos: usize,
}

impl limeshot_dsp::effects::Effect for Lag {
    fn set_params(&mut self, _params: &[f32]) {}
    fn process(&mut self, buf: &mut [f32]) {
        let len = self.line.len();
        for s in buf.iter_mut() {
            std::mem::swap(&mut self.line[self.pos], s);
            self.pos = (self.pos + 1) % len;
        }
    }
    fn reset(&mut self) {}
    fn latency(&self) -> u32 {
        (self.line.len() / 2) as u32
    }
}

/// Three plugin channels play one note together at tick 960 (frame 24 000): A on insert 1
/// through a 1000-frame look-ahead insert, B on insert 2 with nothing, and C on insert 2 with
/// a 300-frame instrument latency. Insert 3 routes into insert 1 (through its lag) and has
/// channel D. Returns the document, the pattern, the channels and the lag effect's id.
fn latency_project() -> (Document, PatternId, Vec<ChannelId>, u32) {
    let mut doc = Document::default();
    doc.execute(Command::SetTempo(Tempo::from_bpm(120.0))).unwrap();
    let pat = doc.project().patterns[0].id;
    let mut channels = Vec::new();
    for (name, track) in [("A", 1u16), ("B", 2), ("C", 2), ("D", 3)] {
        let ch = ChannelId(
            doc.execute(Command::AddChannel {
                name: name.into(),
                instrument: Instrument::Plugin(PluginRef::vst3("/x.vst3", "00", name)),
            })
            .unwrap()
            .0
            .unwrap(),
        );
        doc.execute(Command::SetChannelMixerTrack { channel: ch, track })
            .unwrap();
        doc.execute(Command::AddNote {
            pattern: pat,
            note: NewNote {
                channel: ch,
                key: 60,
                start: 960,
                length: 480,
                velocity: 1.0,
            },
        })
        .unwrap();
        channels.push(ch);
    }
    let lag = doc
        .execute(Command::AddPluginEffect {
            track: 1,
            plugin: PluginRef::vst3("/lag.vst3", "01", "Lag"),
        })
        .unwrap()
        .0
        .unwrap();
    doc.execute(Command::SetRoute {
        from: 3,
        to: 0,
        level: None,
    })
    .unwrap();
    doc.execute(Command::SetRoute {
        from: 3,
        to: 1,
        level: Some(1.0),
    })
    .unwrap();
    (doc, pat, channels, lag)
}

/// Fresh processors for [`latency_project`].
fn latency_processors(channels: &[ChannelId], lag: u32) -> Processors {
    let click = |latency| {
        Box::new(Click {
            latency,
            due: Vec::new(),
            frame: 0,
        }) as Box<dyn PluginInstrument>
    };
    Processors {
        effects: vec![(
            lag,
            Box::new(Lag {
                line: vec![0.0; 2 * 1000],
                pos: 0,
            }),
        )],
        instruments: vec![
            (channels[0].0, click(0)),
            (channels[1].0, click(0)),
            (channels[2].0, click(300)),
            (channels[3].0, click(0)),
        ],
    }
}

/// Frames with a sound.
fn hits(audio: &[f32]) -> Vec<usize> {
    audio
        .chunks(2)
        .enumerate()
        .filter(|(_, f)| f[0].abs() > 1e-6)
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn plugin_latency_is_compensated_so_parallel_paths_line_up() {
    // Every impulse must reach the master on the same frame, and the export starts where the
    // music starts.
    let (doc, pat, channels, lag) = latency_project();
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    for block in [64, 333, 1024] {
        let settings = RenderSettings {
            sample_rate: SR,
            range: ExportRange::Pattern,
            max_tail_secs: 0.0,
            block,
            dither: false,
        };
        let (audio, report) = render_offline_with(snap.clone(), &settings, latency_processors(&channels, lag)).unwrap();
        assert_eq!(
            hits(&audio),
            [24_000],
            "block {block}: every path lines up on the note's frame"
        );
        assert_eq!(
            audio.len() / 2,
            report.music_frames,
            "the compensation delay is trimmed"
        );
    }
}

#[test]
fn split_mixer_tracks_write_aligned_stems_of_the_masters_length() {
    let (doc, pat, channels, lag) = latency_project();
    let snap = Arc::new(compile(doc.project(), pat, &NoSamples));
    let names: Vec<String> = doc.project().mixer.tracks.iter().map(|t| t.name.clone()).collect();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("song.wav");
    for block in [64, 333, 1024] {
        let settings = RenderSettings {
            sample_rate: SR,
            range: ExportRange::Pattern,
            max_tail_secs: 0.0,
            block,
            dither: false,
        };
        let report = export_stems_with(
            snap.clone(),
            &settings,
            ExportFormat::Wav(BitDepth::Float32),
            &path,
            &names,
            latency_processors(&channels, lag),
        )
        .unwrap();
        // Inserts 1-3 sounded; the other inserts stayed silent and are not written.
        let expected: Vec<_> = (1..=3).map(|t| stem_path(&path, t, &names[t])).collect();
        assert_eq!(report.stems, expected, "block {block}");
        let master = limeshot_dsp::load_wav(&path).unwrap();
        assert_eq!(hits(&master.data), [24_000]);
        assert_eq!(master.frames(), report.master.frames);
        for stem in &report.stems {
            let audio = limeshot_dsp::load_wav(stem).unwrap();
            assert_eq!(
                audio.frames(),
                master.frames(),
                "{} has the master's length",
                stem.display()
            );
            assert_eq!(
                hits(&audio.data),
                [24_000],
                "{} lines up with the master",
                stem.display()
            );
        }
        // Insert 1 holds A and, through the route, D; insert 2 holds B and C.
        let peak = |t: usize| {
            let audio = limeshot_dsp::load_wav(&report.stems[t - 1]).unwrap();
            audio.data[2 * 24_000]
        };
        assert!((peak(1) - 2.0 * peak(3)).abs() < 1e-4, "{} vs {}", peak(1), peak(3));
        assert!((peak(2) - 2.0 * peak(3)).abs() < 1e-4);
    }
    // MP3 cannot be split, and nothing is written.
    let mp3 = dir.path().join("song.mp3");
    assert!(matches!(
        export_stems_with(
            snap,
            &RenderSettings::default(),
            ExportFormat::Mp3 { bitrate_kbps: 320 },
            &mp3,
            &names,
            Processors::default()
        ),
        Err(ExportError::StemsNeedWav)
    ));
    assert!(!mp3.exists());
}

#[test]
fn stem_names_are_safe_file_names() {
    let path = std::path::Path::new("/out/My song.wav");
    assert_eq!(
        stem_path(path, 3, "Bass"),
        std::path::Path::new("/out/My song - 03 Bass.wav")
    );
    assert_eq!(
        stem_path(path, 12, " a/b:c? "),
        std::path::Path::new("/out/My song - 12 a_b_c_.wav")
    );
    assert_eq!(stem_path(path, 7, ".."), std::path::Path::new("/out/My song - 07.wav"));
    assert_eq!(
        stem_path(path, 1, "\u{7}"),
        std::path::Path::new("/out/My song - 01.wav")
    );
}

#[test]
fn timed_auditions_release_on_the_audio_clock_while_stopped() {
    let (doc, ch) = project();
    let snap = Arc::new(compile(doc.project(), doc.project().patterns[0].id, &NoSamples));
    for block in [1, 64, 333, 1024] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut renderer = Renderer::new(SR, snap.clone());
        renderer.adopt_instrument(
            ch.0,
            Box::new(Gate {
                held: 0,
                frame: 0,
                log: log.clone(),
            }),
        );
        renderer.set_snapshot(snap.clone());
        renderer.preview_note_for(0, 72, 1.0, 0.05);
        let mut rendered = 0;
        let mut buf = vec![0.0; block * 2];
        while rendered < 4800 {
            renderer.render(&mut buf);
            rendered += block;
        }
        assert_eq!(
            *log.lock().unwrap(),
            vec![(0, true, 72), (2400, false, 72)],
            "block {block}"
        );
    }
}

#[test]
fn insert_effects_receive_current_and_automated_transport() {
    struct TransportProbe {
        current: Transport,
        seen: Arc<Mutex<Vec<Transport>>>,
    }
    impl limeshot_dsp::effects::Effect for TransportProbe {
        fn set_params(&mut self, _: &[f32]) {}
        fn reset(&mut self) {}
        fn set_transport(&mut self, t: &Transport) {
            self.current = *t;
        }
        fn process(&mut self, _: &mut [f32]) {
            self.seen.lock().unwrap().push(self.current);
        }
    }
    for block in [1, 64, 333, 1024] {
        let (mut doc, _) = project();
        doc.execute(Command::SetTempo(Tempo::from_bpm(90.0))).unwrap();
        let effect = doc
            .execute(Command::AddPluginEffect {
                track: 1,
                plugin: PluginRef::vst3("/delay.vst3", "00", "Delay"),
            })
            .unwrap()
            .0
            .unwrap();
        let pat = doc.project().patterns[0].id;
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut renderer = Renderer::new(SR, Arc::new(compile(doc.project(), pat, &NoSamples)));
        renderer.adopt_effect(
            effect,
            Box::new(TransportProbe {
                current: Transport::default(),
                seen: seen.clone(),
            }),
        );
        renderer.rearrange();
        renderer.set_mode(PlayMode::Song);
        renderer.play();
        let mut out = vec![0.0; block * 2];
        renderer.render(&mut out);
        assert_eq!(seen.lock().unwrap()[0].tempo, 90.0);
        doc.execute(Command::AddAutomationClip {
            target: AutomationTarget::Tempo,
            track: 3,
            start: 0,
            length: 3840,
            value: Some(1.0),
            name: None,
        })
        .unwrap();
        let automation = doc.project().automations.last().unwrap().id;
        doc.execute(Command::SetAutomationPoints {
            automation,
            points: vec![AutomationPoint {
                tick: 0,
                value: 1.0,
                tension: 0.0,
                mode: PointMode::Hold,
            }],
        })
        .unwrap();
        renderer.set_snapshot(Arc::new(compile(doc.project(), pat, &NoSamples)));
        renderer.render(&mut out);
        let transport = *seen.lock().unwrap().last().unwrap();
        assert_eq!(
            transport.tempo, 180.0,
            "block {block}: automation reaches the next effect block"
        );
        assert!(transport.playing);
        assert!(transport.ppq_pos >= 0.0 && transport.ppq_pos.is_finite());
        renderer.stop();
        renderer.render(&mut out);
        assert!(!seen.lock().unwrap().last().unwrap().playing);
    }
}

#[test]
fn tempo_ramps_preserve_each_tick_once_and_the_integrated_song_length() {
    let mut doc = Document::default();
    let pat = doc.project().patterns[0].id;
    let channel = ChannelId(
        doc.execute(Command::AddChannel {
            name: "Tick probe".into(),
            instrument: Instrument::Plugin(PluginRef::vst3("/probe.vst3", "00", "Probe")),
        })
        .unwrap()
        .0
        .unwrap(),
    );
    doc.execute(Command::AddNotes {
        pattern: pat,
        notes: (0..3840)
            .map(|tick| NewNote {
                channel,
                key: (tick % 128) as u8,
                start: tick,
                length: 1,
                velocity: 1.0,
            })
            .collect(),
    })
    .unwrap();
    doc.execute(Command::AddClip {
        pattern: pat,
        track: 0,
        start: 0,
        length: Some(3840),
    })
    .unwrap();
    doc.execute(Command::AddAutomationClip {
        target: AutomationTarget::Tempo,
        track: 3,
        start: 0,
        length: 3840,
        value: None,
        name: None,
    })
    .unwrap();
    let automation = doc.project().automations[0].id;
    doc.execute(Command::SetAutomationPoints {
        automation,
        points: vec![
            AutomationPoint {
                tick: 0,
                value: 0.0,
                tension: 0.0,
                mode: PointMode::SingleCurve,
            },
            AutomationPoint {
                tick: 3840,
                value: 1.0,
                tension: 0.0,
                mode: PointMode::SingleCurve,
            },
        ],
    })
    .unwrap();
    let snapshot = Arc::new(compile(doc.project(), pat, &NoSamples));
    let integrated = SR as f64 * 2.0 * 3.0_f64.ln();
    for block in [1, 64, 333, 1024] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let processors = Processors {
            effects: vec![],
            instruments: vec![(
                channel.0,
                Box::new(Gate {
                    held: 0,
                    frame: 0,
                    log: log.clone(),
                }),
            )],
        };
        let (_, report) = render_offline_with(
            snapshot.clone(),
            &RenderSettings {
                sample_rate: SR,
                range: ExportRange::Song,
                max_tail_secs: 0.0,
                block,
                dither: false,
            },
            processors,
        )
        .unwrap();
        let events = log.lock().unwrap();
        let keys: Vec<_> = events.iter().filter(|e| e.1).map(|e| e.2).collect();
        assert_eq!(keys.len(), 3840, "block {block}: exactly one onset per tick");
        assert_eq!(
            keys,
            (0..3840).map(|tick| (tick % 128) as u8).collect::<Vec<_>>(),
            "block {block}: no duplicated or lost tick notes"
        );
        assert!(
            (report.music_frames as f64 - integrated).abs() < 160.0,
            "block {block}: {} frames, integral {integrated}",
            report.music_frames
        );
    }
}
