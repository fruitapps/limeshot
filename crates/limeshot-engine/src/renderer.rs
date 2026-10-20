//! The renderer: transport, scheduling, voices and the mixer.
//!
//! [`Renderer::render`] fills one block of interleaved stereo audio. It is called from the cpal
//! callback for realtime playback and in a loop for offline export, so both paths produce the same
//! samples. It never allocates, locks or does I/O after construction: buffers and the voice pool
//! are sized up front.
//!
//! Scheduling: while playing, the renderer keeps a [`Timeline`] whose origin is the start of the
//! current pass through the program (pattern loop or song). A note at tick `t` starts exactly at
//! `timeline.sample_of_tick(t)`; a block is split at every event so onsets are sample-accurate and
//! independent of the block size.

use crate::voice::{NO_CHANNEL, Voice};
use limeshot_core::{Tick, Timeline};
use limeshot_dsp::{SampleData, pan_gains};
use limeshot_sequencer::{Program, Snapshot};
use std::sync::Arc;

/// Largest block `render` accepts; callers split bigger device buffers.
pub const MAX_BLOCK: usize = 1024;
pub const MAX_VOICES: usize = 128;
/// Events handled per block; more are deferred to the next block (only reachable with absurdly
/// dense programs).
const MAX_EVENTS_PER_BLOCK: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    #[default]
    Pattern,
    Song,
}

/// Per-track peak levels of the last rendered block.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Peak {
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Note {
        channel: u16,
        key: u8,
        velocity: f32,
        end_tick: Tick,
        offset_secs: f64,
        clip: bool,
    },
    Click {
        downbeat: bool,
    },
}

pub struct Renderer {
    sample_rate: u32,
    snapshot: Arc<Snapshot>,
    voices: Vec<Voice>,
    serial: u64,
    /// One stereo buffer (interleaved, `2 * MAX_BLOCK`) per mixer track.
    buses: Vec<Vec<f32>>,
    actions: Vec<(usize, Action)>,
    peaks: Vec<Peak>,
    master_clipped: bool,

    playing: bool,
    mode: PlayMode,
    /// Song mode loops at the end of the arrangement; otherwise playback stops there.
    song_loop: bool,
    metronome: bool,
    /// Engine sample counter (advances whether or not the transport plays).
    now: u64,
    timeline: Timeline,
    /// Where Play started, so Stop can return there.
    start_tick: Tick,
    /// Set when Song mode reached its end without looping.
    finished: bool,
    fx: crate::fx::FxRack,
    /// Peak of the last block's output (for detecting when effect tails have died away).
    last_peak: f32,
    /// Start the audio clips that span the position on the next schedule (after play, locate or
    /// a mode change), part-way through.
    chase: bool,
}

impl Renderer {
    pub fn new(sample_rate: u32, snapshot: Arc<Snapshot>) -> Self {
        let sample_rate = sample_rate.clamp(8_000, 768_000);
        let tempo = snapshot.tempo;
        Self {
            sample_rate,
            snapshot,
            voices: vec![Voice::default(); MAX_VOICES],
            serial: 0,
            buses: (0..limeshot_core::Mixer::MAX_TRACKS)
                .map(|_| vec![0.0; 2 * MAX_BLOCK])
                .collect(),
            actions: Vec::with_capacity(MAX_EVENTS_PER_BLOCK),
            peaks: vec![Peak::default(); limeshot_core::Mixer::MAX_TRACKS],
            master_clipped: false,
            playing: false,
            mode: PlayMode::Pattern,
            song_loop: true,
            metronome: false,
            now: 0,
            timeline: Timeline::new(tempo, sample_rate),
            start_tick: 0,
            finished: false,
            fx: crate::fx::FxRack::new(),
            last_peak: 0.0,
            chase: true,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }

    /// Install a new snapshot and return the old one (the caller drops it off the audio thread).
    pub fn set_snapshot(&mut self, snapshot: Arc<Snapshot>) -> Arc<Snapshot> {
        let tempo_changed = snapshot.tempo != self.snapshot.tempo;
        let channels = snapshot.channels.len();
        let old = std::mem::replace(&mut self.snapshot, snapshot);
        self.fx.arrange(&self.snapshot);
        if tempo_changed {
            // Keep the musical position; continue at the new tempo from here.
            let tick = self.timeline.tick_at_sample(self.now);
            self.timeline = Timeline {
                origin_tick: tick,
                origin_sample: self.now,
                tempo: self.snapshot.tempo,
                sample_rate: self.sample_rate,
            };
            for v in self
                .voices
                .iter_mut()
                .filter(|v| v.sequenced && v.release_at != u64::MAX)
            {
                v.release_at = self.timeline.sample_of_tick(v.end_tick.max(tick));
            }
        }
        for v in &mut self.voices {
            if v.channel != NO_CHANNEL && usize::from(v.channel) >= channels {
                v.env.kill();
            }
        }
        old
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn mode(&self) -> PlayMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: PlayMode) {
        if mode != self.mode {
            self.mode = mode;
            self.release_sequenced();
            self.locate(0);
            self.start_tick = 0;
        }
    }

    pub fn set_song_loop(&mut self, on: bool) {
        self.song_loop = on;
    }

    pub fn set_metronome(&mut self, on: bool) {
        self.metronome = on;
    }

    /// True once Song mode played to the end without looping (cleared by `play`).
    pub fn finished(&self) -> bool {
        self.finished
    }

    fn program(&self) -> &Program {
        match self.mode {
            PlayMode::Pattern => &self.snapshot.pattern,
            PlayMode::Song => &self.snapshot.song,
        }
    }

    /// The loop/end point of the current program; an empty song still runs one bar.
    fn program_length(&self) -> Tick {
        let len = self.program().length;
        if len == 0 {
            self.snapshot.time_signature.ticks_per_bar()
        } else {
            len
        }
    }

    /// The transport position in ticks within the current program.
    pub fn position(&self) -> Tick {
        self.timeline.tick_at_sample(self.now)
    }

    pub fn play(&mut self) {
        if !self.playing {
            self.start_tick = self.position();
            self.playing = true;
            self.finished = false;
            self.chase = true;
        }
    }

    /// Stop and return to where playback started.
    pub fn stop(&mut self) {
        self.playing = false;
        self.release_sequenced();
        let t = self.start_tick;
        self.locate(t);
    }

    /// Stop without moving the playhead.
    pub fn pause(&mut self) {
        self.playing = false;
        self.release_sequenced();
    }

    pub fn locate(&mut self, tick: Tick) {
        let tick = if tick >= self.program_length() { 0 } else { tick };
        self.timeline = Timeline {
            origin_tick: tick,
            origin_sample: self.now,
            tempo: self.snapshot.tempo,
            sample_rate: self.sample_rate,
        };
        if !self.playing {
            self.start_tick = tick;
        }
        self.release_sequenced();
        self.chase = true;
    }

    fn release_sequenced(&mut self) {
        for v in self.voices.iter_mut().filter(|v| v.sequenced) {
            v.release();
        }
    }

    /// Stop every voice immediately (panic button).
    pub fn all_notes_off(&mut self) {
        for v in &mut self.voices {
            v.env.kill();
        }
    }

    fn alloc_voice(&mut self, mut v: Voice) {
        self.serial += 1;
        v.serial = self.serial;
        let slot = match self.voices.iter().position(|v| !v.is_active()) {
            Some(i) => i,
            None => {
                // Steal the oldest voice.
                let mut oldest = 0;
                for (i, x) in self.voices.iter().enumerate() {
                    if x.serial < self.voices[oldest].serial {
                        oldest = i;
                    }
                }
                oldest
            }
        };
        self.voices[slot] = v;
    }

    /// Audition a channel's instrument (piano keys, step clicks).
    pub fn preview_note_on(&mut self, channel: u16, key: u8, velocity: f32) {
        let Some(ch) = self.snapshot.channels.get(usize::from(channel)) else {
            return;
        };
        if let Some(mut v) = Voice::for_instrument(&ch.instrument, ch.root_key, key, velocity, self.sample_rate as f32)
        {
            v.channel = channel;
            self.alloc_voice(v);
        }
    }

    pub fn preview_note_off(&mut self, channel: u16, key: u8) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| !v.sequenced && v.channel == channel && v.key == key)
        {
            v.release();
        }
    }

    /// Play a sample from the browser into the master.
    pub fn preview_sample(&mut self, sample: Arc<SampleData>) {
        self.stop_sample_preview();
        if let Some(v) = Voice::preview(sample, self.sample_rate as f32) {
            self.alloc_voice(v);
        }
    }

    pub fn stop_sample_preview(&mut self) {
        for v in self
            .voices
            .iter_mut()
            .filter(|v| !v.sequenced && v.channel == NO_CHANNEL)
        {
            v.release();
        }
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Peak levels of the last block, per mixer track (index 0 = master).
    pub fn peaks(&self) -> &[Peak] {
        &self.peaks[..self.snapshot.mixer.len().min(self.peaks.len())]
    }

    /// Whether the master exceeded full scale since the last call.
    pub fn take_master_clipped(&mut self) -> bool {
        std::mem::take(&mut self.master_clipped)
    }

    /// Render `out.len() / 2` stereo frames (at most [`MAX_BLOCK`]) of interleaved audio.
    pub fn render(&mut self, out: &mut [f32]) {
        let frames = (out.len() / 2).min(MAX_BLOCK);
        let tracks = self.snapshot.mixer.len().clamp(1, self.buses.len());
        for b in &mut self.buses[..tracks] {
            b[..2 * frames].fill(0.0);
        }

        let mut done = 0;
        while done < frames {
            let chunk = if self.playing {
                self.schedule(frames - done)
            } else {
                frames - done
            };
            let chunk = chunk.max(1).min(frames - done);
            // Render voices between actions; `actions` holds offsets relative to `done`.
            let mut cursor = 0;
            let actions = std::mem::take(&mut self.actions);
            for &(offset, action) in &actions {
                let offset = offset.min(chunk).max(cursor);
                self.render_voices(done + cursor, offset - cursor, self.now + cursor as u64);
                cursor = offset;
                self.start(action);
            }
            self.render_voices(done + cursor, chunk - cursor, self.now + cursor as u64);
            self.actions = actions;
            self.actions.clear();
            done += chunk;
            self.now += chunk as u64;
            if self.playing {
                self.wrap_if_at_end();
            } else {
                // Stopped: the playhead stays where it is.
                self.timeline.origin_sample = self.now;
            }
        }

        self.mix(tracks, frames, out);
    }

    /// Collect the actions due in the next `max` frames (up to the end of the current pass) into
    /// `self.actions` and return how many frames this chunk covers.
    fn schedule(&mut self, max: usize) -> usize {
        let len = self.program_length();
        let s0 = self.now;
        let pass_end = self.timeline.sample_of_tick(len);
        if pass_end <= s0 {
            return max;
        }
        let chunk_end = (s0 + max as u64).min(pass_end);
        let t_lo = self.timeline.first_tick_at_or_after(s0);
        let t_hi = self.timeline.first_tick_at_or_after(chunk_end).min(len);
        let tl = self.timeline;
        let mut actions = std::mem::take(&mut self.actions);
        if std::mem::take(&mut self.chase) {
            for e in self.program().range(0, t_lo) {
                if e.clip && e.end() > t_lo && actions.len() < MAX_EVENTS_PER_BLOCK {
                    actions.push((
                        0,
                        Action::Note {
                            channel: e.channel,
                            key: e.key,
                            velocity: e.velocity,
                            end_tick: e.end(),
                            offset_secs: e.offset_secs + tl.seconds(t_lo - e.tick),
                            clip: true,
                        },
                    ));
                }
            }
        }
        for e in self.program().range(t_lo, t_hi) {
            if actions.len() >= MAX_EVENTS_PER_BLOCK {
                break;
            }
            let offset = tl.sample_of_tick(e.tick).saturating_sub(s0) as usize;
            actions.push((
                offset,
                Action::Note {
                    channel: e.channel,
                    key: e.key,
                    velocity: e.velocity,
                    end_tick: e.tick + e.length,
                    offset_secs: e.offset_secs,
                    clip: e.clip,
                },
            ));
        }
        if self.metronome {
            let sig = self.snapshot.time_signature;
            let beat = sig.ticks_per_beat();
            let bar = sig.ticks_per_bar();
            let mut t = t_lo.div_ceil(beat) * beat;
            while t < t_hi && actions.len() < MAX_EVENTS_PER_BLOCK {
                let offset = tl.sample_of_tick(t).saturating_sub(s0) as usize;
                actions.push((
                    offset,
                    Action::Click {
                        downbeat: t.is_multiple_of(bar),
                    },
                ));
                t += beat;
            }
        }
        actions.sort_by_key(|a| a.0);
        self.actions = actions;
        (chunk_end - s0) as usize
    }

    fn wrap_if_at_end(&mut self) {
        let len = self.program_length();
        let pass_end = self.timeline.sample_of_tick(len);
        if self.now < pass_end {
            return;
        }
        if self.mode == PlayMode::Song && !self.song_loop {
            self.playing = false;
            self.finished = true;
            self.release_sequenced();
            self.timeline = Timeline {
                origin_tick: 0,
                origin_sample: self.now,
                tempo: self.snapshot.tempo,
                sample_rate: self.sample_rate,
            };
            self.start_tick = 0;
            return;
        }
        // Notes do not ring across the loop point, except one-shot samples.
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.sequenced && v.release_at != u64::MAX)
        {
            v.release();
        }
        self.timeline = Timeline {
            origin_tick: 0,
            origin_sample: pass_end,
            tempo: self.snapshot.tempo,
            sample_rate: self.sample_rate,
        };
    }

    fn start(&mut self, action: Action) {
        let sr = self.sample_rate as f32;
        match action {
            Action::Note {
                channel,
                key,
                velocity,
                end_tick,
                offset_secs,
                clip,
            } => {
                let Some(ch) = self.snapshot.channels.get(usize::from(channel)) else {
                    return;
                };
                let one_shot = !clip
                    && matches!(
                        ch.instrument,
                        limeshot_sequencer::InstrumentState::Sampler { one_shot: true, .. }
                    );
                let Some(mut v) = Voice::for_instrument(&ch.instrument, ch.root_key, key, velocity, sr) else {
                    return;
                };
                if offset_secs > 0.0 {
                    v.skip(offset_secs, sr);
                    if !v.is_active() {
                        return;
                    }
                }
                v.channel = channel;
                v.sequenced = true;
                v.end_tick = end_tick;
                v.release_at = if one_shot {
                    u64::MAX
                } else {
                    self.timeline.sample_of_tick(end_tick)
                };
                self.alloc_voice(v);
            }
            Action::Click { downbeat } => self.alloc_voice(Voice::click(downbeat, sr)),
        }
    }

    /// Render all voices for `n` frames into the buses at frame `at` of the block; `base` is the
    /// engine sample of the first frame.
    fn render_voices(&mut self, at: usize, n: usize, base: u64) {
        if n == 0 {
            return;
        }
        let snapshot = &self.snapshot;
        let tracks = snapshot.mixer.len().clamp(1, self.buses.len());
        for v in self.voices.iter_mut().filter(|v| v.is_active()) {
            let (track, gl, gr, audible) = match snapshot.channels.get(usize::from(v.channel)) {
                Some(ch) => {
                    let (pl, pr) = pan_gains(ch.pan);
                    (
                        ch.mixer_track.min(tracks - 1),
                        ch.volume * pl,
                        ch.volume * pr,
                        ch.audible || !v.sequenced,
                    )
                }
                None => (0, 1.0, 1.0, true),
            };
            let bus = &mut self.buses[track];
            for i in 0..n {
                if base + i as u64 >= v.release_at {
                    v.release_at = u64::MAX;
                    v.release();
                }
                let (l, r) = v.next_frame();
                if audible {
                    let j = 2 * (at + i);
                    bus[j] += l * gl;
                    bus[j + 1] += r * gr;
                }
                if !v.is_active() {
                    break;
                }
            }
        }
    }

    /// Take ownership of an effect processor built on the control thread (see [`crate::fx`]). It
    /// is placed by the next snapshot; call [`Renderer::rearrange`] to place it now.
    pub fn adopt_effect(&mut self, id: u32, fx: Box<dyn limeshot_dsp::effects::Effect>) {
        self.fx.adopt(id, fx);
    }

    /// Drop every effect processor (a different project was opened).
    pub fn clear_effects(&mut self) {
        self.fx.clear();
    }

    /// Re-apply the current snapshot's effect layout (after adopting processors).
    pub fn rearrange(&mut self) {
        let snapshot = self.snapshot.clone();
        self.fx.arrange(&snapshot);
    }

    /// A processor that is no longer used; the caller drops it off the audio thread.
    pub fn take_retired_effect(&mut self) -> Option<Box<dyn limeshot_dsp::effects::Effect>> {
        self.fx.take_retired()
    }

    pub fn keep_retired_effect(&mut self, fx: Box<dyn limeshot_dsp::effects::Effect>) {
        self.fx.keep_retired(fx);
    }

    /// Whether effects may still be producing sound (delay and reverb tails).
    pub fn is_ringing(&self) -> bool {
        self.fx.has_effects() && self.last_peak > 1e-4
    }

    fn mix(&mut self, tracks: usize, frames: usize, out: &mut [f32]) {
        // Inserts in routing order: effects, fader and pan in place, then add the result into
        // every destination (the master by default). Sources come before their destinations.
        let snapshot = self.snapshot.clone();
        for &i in &snapshot.mix_order {
            if i == 0 || i >= tracks {
                continue;
            }
            let Some(t) = snapshot.mixer.get(i) else { continue };
            self.fx.process(i, &mut self.buses[i][..2 * frames]);
            let (pl, pr) = pan_gains(t.pan);
            let (gl, gr) = if t.audible {
                (t.gain * pl, t.gain * pr)
            } else {
                (0.0, 0.0)
            };
            let mut peak = Peak::default();
            {
                let bus = &mut self.buses[i];
                for f in 0..frames {
                    bus[2 * f] *= gl;
                    bus[2 * f + 1] *= gr;
                    peak.left = peak.left.max(bus[2 * f].abs());
                    peak.right = peak.right.max(bus[2 * f + 1].abs());
                }
            }
            self.peaks[i] = peak;
            for &(to, level) in &t.routes {
                if to == i || to >= tracks {
                    continue;
                }
                let (src, dst) = if i < to {
                    let (a, b) = self.buses.split_at_mut(to);
                    (&a[i], &mut b[0])
                } else {
                    let (a, b) = self.buses.split_at_mut(i);
                    (&b[0], &mut a[to])
                };
                for (d, s) in dst[..2 * frames].iter_mut().zip(&src[..2 * frames]) {
                    *d += s * level;
                }
            }
        }
        let master = &mut self.buses[0];
        self.fx.process(0, &mut master[..2 * frames]);
        let t = &self.snapshot.mixer[0];
        let (pl, pr) = pan_gains(t.pan);
        let (gl, gr) = if t.audible {
            (t.gain * pl, t.gain * pr)
        } else {
            (0.0, 0.0)
        };
        let mut peak = Peak::default();
        for f in 0..frames {
            let l = master[2 * f] * gl;
            let r = master[2 * f + 1] * gr;
            peak.left = peak.left.max(l.abs());
            peak.right = peak.right.max(r.abs());
            out[2 * f] = if l.is_finite() { l } else { 0.0 };
            out[2 * f + 1] = if r.is_finite() { r } else { 0.0 };
        }
        if peak.left > 1.0 || peak.right > 1.0 {
            self.master_clipped = true;
        }
        self.last_peak = peak.left.max(peak.right);
        self.peaks[0] = peak;
        out[2 * frames..].fill(0.0);
    }
}
