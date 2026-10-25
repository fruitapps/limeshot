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

use limeshot_core::{Tick, Timeline};
use limeshot_dsp::voice::{NO_CHANNEL, Voice};
use limeshot_dsp::{SampleData, pan_gains};
use limeshot_sequencer::{LaneTarget, Program, Snapshot};
use std::sync::Arc;

/// Largest block `render` accepts; callers split bigger device buffers.
pub const MAX_BLOCK: usize = 1024;
pub const MAX_VOICES: usize = 128;
/// Channels whose volume and pan automation can drive (more are played at their set values).
pub const MAX_AUTOMATED_CHANNELS: usize = 1024;
/// Automation is re-evaluated at least this often (frames).
const AUTOMATION_INTERVAL: usize = 64;
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
        pan: f32,
        playback: Option<limeshot_dsp::ClipPlayback>,
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
    /// Song mode: a time selection that playback loops (FL Studio's Playlist time selection).
    loop_range: Option<(Tick, Tick)>,
    metronome: bool,
    /// Engine sample counter (advances whether or not the transport plays).
    now: u64,
    timeline: Timeline,
    /// Where Play started, so Stop can return there.
    start_tick: Tick,
    /// Set when Song mode reached its end without looping.
    finished: bool,
    fx: crate::fx::FxRack,
    /// Plugin instruments, by channel.
    instruments: crate::plugins::InstrumentRack,
    /// Each mixer track's gain reduction in dB in the last block (its compressors).
    reductions: Vec<f32>,
    /// Peak of the last block's output (for detecting when effect tails have died away).
    last_peak: f32,
    /// Clear the effects' tails at this engine sample (once silenced voices have faded).
    fx_reset_at: Option<u64>,
    /// Start the audio clips that span the position on the next schedule (after play, locate or
    /// a mode change), part-way through.
    chase: bool,
    /// Channel (volume, pan) and mixer track (gain, pan) in effect: the snapshot's values,
    /// overridden by automation in Song mode.
    chan_mix: Vec<(f32, f32)>,
    /// Per-frame hosted channel gains, captured at each automation interval.
    chan_gains: Vec<(f32, f32)>,
    track_mix: Vec<(f32, f32)>,
    /// Plugin delay compensation.
    pdc: crate::pdc::Pdc,
    /// Blocks until plugin latencies are checked again.
    pdc_check: u32,
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
            loop_range: None,
            metronome: false,
            now: 0,
            timeline: Timeline::new(tempo, sample_rate),
            start_tick: 0,
            finished: false,
            fx: crate::fx::FxRack::new(),
            instruments: crate::plugins::InstrumentRack::new(MAX_BLOCK),
            reductions: vec![0.0; limeshot_core::Mixer::MAX_TRACKS],
            last_peak: 0.0,
            fx_reset_at: None,
            chase: true,
            chan_mix: vec![(0.0, 0.0); MAX_AUTOMATED_CHANNELS],
            chan_gains: vec![(1.0, 1.0); crate::MAX_PLUGIN_CHANNELS * MAX_BLOCK],
            track_mix: vec![(1.0, 0.0); limeshot_core::Mixer::MAX_TRACKS],
            pdc: crate::pdc::Pdc::new(),
            pdc_check: 0,
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
        self.instruments.arrange(&self.snapshot);
        self.plan_pdc(true);
        if tempo_changed {
            self.retime(self.snapshot.tempo);
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

    /// Continue at `tempo` from the current sample, keeping the musical position: scheduled
    /// note ends move with it.
    fn retime(&mut self, tempo: limeshot_core::Tempo) {
        let tick = self.timeline.tick_at_sample(self.now);
        self.timeline = self.timeline.retimed(self.now, tempo);
        for v in self
            .voices
            .iter_mut()
            .filter(|v| v.sequenced && v.release_at != u64::MAX)
        {
            v.release_at = self.timeline.sample_of_tick(v.end_tick.max(tick));
        }
        let tl = self.timeline;
        self.instruments.retime(|t| tl.sample_of_tick(t.max(tick)));
    }

    /// The tempo the transport runs at now (tempo automation included).
    pub fn current_tempo(&self) -> limeshot_core::Tempo {
        self.timeline.tempo
    }

    /// Frames from now until tick `t` at the current tempo (0 when it has passed).
    pub fn frames_until(&self, t: Tick) -> u64 {
        self.timeline.sample_of_tick(t).saturating_sub(self.now)
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

    /// Loop `start..end` in Song mode (`None` or an empty range: the whole song).
    pub fn set_loop_range(&mut self, range: Option<(Tick, Tick)>) {
        self.loop_range = range.filter(|(a, b)| a < b);
    }

    fn active_loop(&self) -> Option<(Tick, Tick)> {
        match self.mode {
            PlayMode::Song => self.loop_range,
            PlayMode::Pattern => None,
        }
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
        if let Some((_, end)) = self.active_loop() {
            return end;
        }
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
            // Start from silence: earlier reverb tails and auditions do not ring into the song.
            self.silence();
            self.reset_effects();
            self.start_tick = self.position();
            self.playing = true;
            self.finished = false;
            self.chase = true;
        }
    }

    /// Stop and return to where playback started. Stopping while stopped silences everything
    /// (sample previews, auditions, effect tails).
    pub fn stop(&mut self) {
        if !self.playing {
            self.silence();
        }
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
        let tick = if tick >= self.program_length() {
            self.active_loop().map_or(0, |r| r.0)
        } else {
            tick
        };
        self.timeline = Timeline {
            origin_tick: tick,
            origin_sample: self.now,
            sub_tick: 0,
            origin_first_tick: tick,
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
        self.instruments.release_sequenced();
    }

    /// Silence everything: every voice (sequenced notes, auditions, sample previews) fades out
    /// in 5 ms, plugins get their note-offs and are reset, and the effects' tails are cleared
    /// once the fade is over (so it does not ring on in a reverb).
    pub fn silence(&mut self) {
        for v in self.voices.iter_mut().filter(|v| v.is_active()) {
            v.env.kill();
        }
        let transport = self.transport();
        self.instruments.silence(&transport);
        self.fx_reset_at = Some(self.now + u64::from(self.sample_rate) / 100);
    }

    fn reset_effects(&mut self) {
        self.fx_reset_at = None;
        self.fx.reset();
        self.pdc.clear();
    }

    /// Stop every voice immediately (panic button).
    pub fn all_notes_off(&mut self) {
        for v in &mut self.voices {
            v.env.kill();
        }
        self.instruments.release_all();
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

    /// Audition a channel's instrument (piano keys, step clicks) without a scheduled release.
    /// `channel` indexes the current snapshot; an out-of-range index is ignored.
    /// Use `preview_note_off` to release the note.
    pub fn preview_note_on(&mut self, channel: u16, key: u8, velocity: f32) {
        self.preview_note_for(channel, key, velocity, f64::INFINITY);
    }

    /// Audition a channel's instrument with a release deadline on the audio clock, even while stopped.
    ///
    /// `channel` indexes the current snapshot; an out-of-range index is ignored. Finite `seconds`
    /// are clamped to at least zero and rounded to the nearest sample frame; nonfinite values schedule no
    /// release. The instrument's release tail may continue after the deadline.
    pub fn preview_note_for(&mut self, channel: u16, key: u8, velocity: f32, seconds: f64) {
        let off_at = if seconds.is_finite() {
            self.now
                .saturating_add((seconds.max(0.0) * self.sample_rate as f64).round() as u64)
        } else {
            u64::MAX
        };
        let Some(ch) = self.snapshot.channels.get(usize::from(channel)) else {
            return;
        };
        if let limeshot_sequencer::InstrumentState::Plugin { channel: id } = ch.instrument {
            let at = (0, self.now);
            self.instruments
                .note_on(usize::from(channel), id, key, velocity, at, off_at, 0, false, 0.0, None);
        }
    }

    /// Audition a Playlist source window through its hosted sampler, including offsets and fades.
    pub fn preview_clip(&mut self, channel: u16, playback: limeshot_dsp::ClipPlayback) {
        let Some(ch) = self.snapshot.channels.get(usize::from(channel)) else {
            return;
        };
        if let limeshot_sequencer::InstrumentState::Plugin { channel: id } = ch.instrument {
            let off_at = self
                .now
                .saturating_add((playback.remaining_secs.max(0.0) * f64::from(self.sample_rate)).round() as u64);
            self.instruments.note_on(
                usize::from(channel),
                id,
                ch.root_key,
                1.0,
                (0, self.now),
                off_at,
                0,
                false,
                0.0,
                Some(playback),
            );
        }
    }

    pub fn preview_note_off(&mut self, channel: u16, key: u8) {
        if let Some(limeshot_sequencer::InstrumentState::Plugin { channel: id }) =
            self.snapshot.channels.get(usize::from(channel)).map(|c| &c.instrument)
        {
            self.instruments.preview_off(*id, key);
        }
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
        self.voices.iter().filter(|v| v.is_active()).count() + self.instruments.active_voices()
    }

    /// Peak levels of the last block, per mixer track (index 0 = master).
    /// Each mixer track's gain reduction in dB during the last block (for metering).
    pub fn gain_reductions(&self) -> &[f32] {
        &self.reductions[..self.snapshot.mixer.len().min(self.reductions.len())]
    }

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
        let block_start = self.now;
        let mut transport = self.transport();
        let tracks = self.snapshot.mixer.len().clamp(1, self.buses.len());
        for b in &mut self.buses[..tracks] {
            b[..2 * frames].fill(0.0);
        }

        if self.fx_reset_at.is_some_and(|at| self.now >= at) {
            self.reset_effects();
        }
        let automate = self.mode == PlayMode::Song && !self.snapshot.automation.is_empty();
        let mut done = 0;
        while done < frames {
            self.apply_controls(done as u32);
            if done == 0 {
                transport = self.transport();
            }
            let max = if automate {
                (frames - done).min(AUTOMATION_INTERVAL)
            } else {
                frames - done
            };
            let chunk = if self.playing { self.schedule(max) } else { max };
            let chunk = chunk.max(1).min(frames - done);
            // Render voices between actions; `actions` holds offsets relative to `done`.
            let mut cursor = 0;
            let actions = std::mem::take(&mut self.actions);
            for &(offset, action) in &actions {
                let offset = offset.min(chunk).max(cursor);
                self.render_voices(done + cursor, offset - cursor, self.now + cursor as u64);
                cursor = offset;
                self.start(action, (done + cursor, self.now + cursor as u64));
            }
            self.render_voices(done + cursor, chunk - cursor, self.now + cursor as u64);
            self.actions = actions;
            self.actions.clear();
            for (i, ch) in self
                .snapshot
                .channels
                .iter()
                .enumerate()
                .take(crate::MAX_PLUGIN_CHANNELS)
            {
                let (volume, pan) = self.chan_mix.get(i).copied().unwrap_or((ch.volume, ch.pan));
                let native_pan = self.instruments.channel_pan(i, done as u32, pan);
                let (pl, pr) = pan_gains(if native_pan { 0.0 } else { pan });
                self.chan_gains[i * MAX_BLOCK + done..i * MAX_BLOCK + done + chunk].fill((volume * pl, volume * pr));
            }
            done += chunk;
            self.now += chunk as u64;
            if self.playing {
                self.wrap_if_at_end();
            } else {
                // Stopped: the playhead stays where it is.
                self.timeline.origin_sample = self.now;
            }
        }

        // Plugins may report a new latency at any time (after loading a preset, say).
        if self.pdc_check == 0 {
            self.plan_pdc(false);
            self.pdc_check = 16;
        }
        self.pdc_check -= 1;
        for t in 0..tracks {
            self.pdc.voices(t, &mut self.buses[t][..2 * frames]);
        }
        self.render_instruments(block_start, frames, tracks, &transport);
        self.fx.set_transport(&transport);
        self.mix(tracks, frames, out);
    }

    /// The transport as plugins see it, at the current position.
    fn transport(&self) -> limeshot_dsp::Transport {
        let tick = self.position();
        let bpm = f64::from(self.timeline.tempo.milli_bpm()) / 1000.0;
        let quarters = tick as f64 / limeshot_core::PPQ as f64;
        let sig = self.snapshot.time_signature;
        limeshot_dsp::Transport {
            playing: self.playing,
            tempo: bpm,
            sample_pos: (quarters * 60.0 / bpm * f64::from(self.sample_rate)) as i64,
            ppq_pos: quarters,
            time_sig: (i32::from(sig.numerator), i32::from(sig.denominator)),
        }
    }

    /// Render the plugin channels for the whole block into their mixer buses. Their notes were
    /// queued at exact frame offsets; volume and pan follow the captured automation intervals.
    fn render_instruments(&mut self, start: u64, frames: usize, tracks: usize, transport: &limeshot_dsp::Transport) {
        let snapshot = &self.snapshot;
        let chan_gains = &self.chan_gains;
        let pdc = &mut self.pdc;
        self.instruments
            .render(start, frames, transport, &mut self.buses, pdc, |i, frame| {
                let ch = snapshot.channels.get(i)?;
                if !ch.audible {
                    return None;
                }
                let (gl, gr) = chan_gains[i * MAX_BLOCK + frame];
                Some((ch.mixer_track.min(tracks - 1), gl, gr))
            });
    }

    /// Load channel and mixer settings from the snapshot and apply automation at the current
    /// position (Song mode only). `offset` is the frame in this block (plugin automation points).
    fn apply_controls(&mut self, offset: u32) {
        let snapshot = self.snapshot.clone();
        for (m, ch) in self.chan_mix.iter_mut().zip(&snapshot.channels) {
            *m = (ch.volume, ch.pan);
        }
        for (m, t) in self.track_mix.iter_mut().zip(&snapshot.mixer) {
            *m = (t.gain, t.pan);
        }
        // Tempo automation re-times the transport (Song mode only, like all automation).
        let tick = self.position();
        let tempo = snapshot
            .automation
            .iter()
            .filter(|_| self.mode == PlayMode::Song)
            .find(|l| l.target == LaneTarget::Tempo)
            .and_then(|l| l.value_at(tick))
            .map_or(snapshot.tempo, |bpm| limeshot_core::Tempo::from_bpm(f64::from(bpm)));
        if tempo != self.timeline.tempo {
            self.retime(tempo);
        }
        if self.mode != PlayMode::Song {
            return;
        }
        for lane in &snapshot.automation {
            let Some(v) = lane.value_at(tick) else { continue };
            match lane.target {
                LaneTarget::Tempo => {}
                LaneTarget::ChannelVolume(i) => {
                    if let Some(m) = self.chan_mix.get_mut(i) {
                        m.0 = v;
                    }
                }
                LaneTarget::ChannelPan(i) => {
                    if let Some(m) = self.chan_mix.get_mut(i) {
                        m.1 = v;
                    }
                }
                LaneTarget::MixerVolume(i) => {
                    if let Some(m) = self.track_mix.get_mut(i) {
                        m.0 = v;
                    }
                }
                LaneTarget::MixerPan(i) => {
                    if let Some(m) = self.track_mix.get_mut(i) {
                        m.1 = v;
                    }
                }
                LaneTarget::EffectParam { track, effect, index } => self.fx.set_param(track, effect, index, v),
                LaneTarget::ChannelPluginParam { channel, id } => {
                    self.instruments.set_plugin_param(channel, id, offset, v)
                }
                LaneTarget::EffectPluginParam { track, effect, id } => {
                    self.fx.set_plugin_param(track, effect, id, offset, v)
                }
            }
        }
    }

    /// The (volume, pan) a channel plays at now, automation included.
    pub fn channel_mix(&self, channel: usize) -> Option<(f32, f32)> {
        self.snapshot.channels.get(channel)?;
        self.chan_mix.get(channel).copied()
    }

    /// The (gain, pan) a mixer track plays at now, automation included.
    pub fn track_mix(&self, track: usize) -> Option<(f32, f32)> {
        self.snapshot.mixer.get(track)?;
        self.track_mix.get(track).copied()
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
                            pan: e.pan,
                            playback: Some(limeshot_dsp::ClipPlayback {
                                offset_secs: e.offset_secs,
                                elapsed_secs: Timeline::new(tl.tempo, self.sample_rate)
                                    .sample_of_tick(t_lo.saturating_sub(e.tick))
                                    as f64
                                    / f64::from(self.sample_rate),
                                remaining_secs: tl.sample_of_tick(e.end()).saturating_sub(s0) as f64
                                    / f64::from(self.sample_rate),
                                fade_in_secs: e.fade_in_secs,
                                fade_out_secs: e.fade_out_secs,
                            }),
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
                    pan: e.pan,
                    playback: e.clip.then_some(limeshot_dsp::ClipPlayback {
                        offset_secs: e.offset_secs,
                        elapsed_secs: 0.0,
                        remaining_secs: tl.sample_of_tick(e.end()).saturating_sub(tl.sample_of_tick(e.tick)) as f64
                            / f64::from(self.sample_rate),
                        fade_in_secs: e.fade_in_secs,
                        fade_out_secs: e.fade_out_secs,
                    }),
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
        if self.mode == PlayMode::Song && !self.song_loop && self.loop_range.is_none() {
            self.playing = false;
            self.finished = true;
            self.release_sequenced();
            self.timeline = Timeline {
                origin_tick: 0,
                origin_sample: self.now,
                sub_tick: 0,
                origin_first_tick: 0,
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
        self.instruments.release_sequenced();
        let restart = self.active_loop().map_or(0, |r| r.0);
        self.timeline = Timeline {
            origin_tick: restart,
            origin_sample: pass_end,
            sub_tick: 0,
            origin_first_tick: restart,
            tempo: self.snapshot.tempo,
            sample_rate: self.sample_rate,
        };
        // A loop range can start inside a clip that began earlier.
        self.chase = restart > 0;
    }

    /// Start `action` at block frame `at.0`, engine sample `at.1`.
    fn start(&mut self, action: Action, at: (usize, u64)) {
        let sr = self.sample_rate as f32;
        match action {
            Action::Note {
                channel,
                key,
                velocity,
                end_tick,
                pan,
                playback,
            } => {
                let Some(ch) = self.snapshot.channels.get(usize::from(channel)) else {
                    return;
                };
                if let limeshot_sequencer::InstrumentState::Plugin { channel: id } = ch.instrument {
                    let off_at = self.timeline.sample_of_tick(end_tick);
                    self.instruments.note_on(
                        usize::from(channel),
                        id,
                        key,
                        velocity,
                        at,
                        off_at,
                        end_tick,
                        true,
                        pan,
                        playback,
                    );
                }
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
            let i = usize::from(v.channel);
            let (track, gl, gr, audible) = match snapshot.channels.get(i) {
                Some(ch) => {
                    let (volume, pan) = self.chan_mix.get(i).copied().unwrap_or((ch.volume, ch.pan));
                    let (pl, pr) = pan_gains((pan + v.pan).clamp(-1.0, 1.0));
                    (
                        ch.mixer_track.min(tracks - 1),
                        volume * pl,
                        volume * pr,
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

    /// Drop every effect processor and plugin instrument (a different project was opened).
    pub fn clear_effects(&mut self) {
        self.fx.clear();
        self.instruments.clear();
    }

    /// Re-apply the current snapshot's effect and instrument layout (after adopting processors).
    pub fn rearrange(&mut self) {
        let snapshot = self.snapshot.clone();
        self.fx.arrange(&snapshot);
        self.instruments.arrange(&snapshot);
        self.plan_pdc(true);
    }

    /// Plan delay compensation for the current snapshot and plugin latencies (`force`: the
    /// layout changed; otherwise only when a latency did).
    fn plan_pdc(&mut self, force: bool) {
        let (fx, instruments) = (&self.fx, &self.instruments);
        self.pdc.plan(
            &self.snapshot,
            |t| fx.track_latency(t),
            |i| instruments.latency(i),
            force,
        );
    }

    /// How far the output lags the transport, in frames: the longest plugin latency on the way
    /// to the master (plugin delay compensation delays every other path to match).
    pub fn latency(&self) -> u32 {
        self.pdc.total()
    }

    /// Take ownership of the plugin instrument of channel `channel` (its id), built on the
    /// control thread. It plays from the next snapshot or [`Renderer::rearrange`]; it replaces
    /// any instrument adopted for the channel before.
    pub fn adopt_instrument(&mut self, channel: u32, inst: Box<dyn limeshot_dsp::Instrument>) {
        self.instruments.adopt(channel, inst);
    }

    /// An instrument that is no longer used; the caller drops it off the audio thread.
    pub fn take_retired_instrument(&mut self) -> Option<Box<dyn limeshot_dsp::Instrument>> {
        self.instruments.take_retired()
    }

    pub fn keep_retired_instrument(&mut self, inst: Box<dyn limeshot_dsp::Instrument>) {
        self.instruments.keep_retired(inst);
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
        (self.fx.has_effects() || self.instruments.has_instruments()) && self.last_peak > 1e-4
    }

    /// Track `i`'s hosted insert effects over its bus. A sidechained effect reads the
    /// (post-fader) output of a track that routes to `i`; such a track was processed earlier.
    fn track_fx(&mut self, i: usize, frames: usize, mixer: &[limeshot_sequencer::MixerTrackState]) {
        // Taking the bus out (and putting it back) moves pointers only: no allocation.
        let mut bus = std::mem::take(&mut self.buses[i]);
        let n = (2 * frames).min(bus.len());
        let buses = &self.buses;
        let key = |s: usize| -> Option<&[f32]> {
            let routes_here = mixer.get(s).is_some_and(|src| src.routes.iter().any(|r| r.0 == i));
            if routes_here {
                buses.get(s).and_then(|b| b.get(..n))
            } else {
                None
            }
        };
        let reduction = self.fx.process(i, &mut bus[..n], &key);
        if let Some(r) = self.reductions.get_mut(i) {
            *r = reduction;
        }

        self.buses[i] = bus;
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
            self.track_fx(i, frames, &snapshot.mixer);
            let (gain, pan) = self.track_mix.get(i).copied().unwrap_or((t.gain, t.pan));
            let (pl, pr) = pan_gains(pan);
            let (gl, gr) = if t.audible { (gain * pl, gain * pr) } else { (0.0, 0.0) };
            // Routes read the pre-fader signal: a post-fader route applies the fader and pan, a
            // pre-fader send only its level (a muted track sends nothing either way). The
            // delay-compensation rings hold the pre-fader signal for the same reason.
            self.pdc.record(i, &self.buses[i][..2 * frames]);
            let on = if t.audible { 1.0 } else { 0.0 };
            for &(to, level, pre) in &t.routes {
                if to == i || to >= tracks {
                    continue;
                }
                let gains = if pre {
                    (level * on, level * on)
                } else {
                    (level * gl, level * gr)
                };
                let (src, dst) = if i < to {
                    let (a, b) = self.buses.split_at_mut(to);
                    (&a[i], &mut b[0])
                } else {
                    let (a, b) = self.buses.split_at_mut(i);
                    (&b[0], &mut a[to])
                };
                self.pdc.route(i, to, &src[..2 * frames], &mut dst[..2 * frames], gains);
            }
            // The bus ends post-fader: the meters and sidechain keys read it.
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
        }
        self.track_fx(0, frames, &snapshot.mixer);
        let master = &mut self.buses[0];
        let t = &self.snapshot.mixer[0];
        let (gain, pan) = self.track_mix.first().copied().unwrap_or((t.gain, t.pan));
        let (pl, pr) = pan_gains(pan);
        let (gl, gr) = if t.audible { (gain * pl, gain * pr) } else { (0.0, 0.0) };
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
