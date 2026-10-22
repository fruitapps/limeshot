//! Realtime playback through the system audio device, and background sample loading.
//!
//! Threads:
//! * The **control thread** (the UI) owns the [`Player`]. It sends [`Msg`]s through a bounded,
//!   lock-free SPSC ring (`rtrb`) and reads playback state from atomics in [`Status`].
//! * The **audio thread** (the cpal callback) owns the [`Renderer`]. Each callback drains pending
//!   messages without blocking, renders, and publishes position, meters and load. Snapshots it
//!   replaces travel back through a second ring and are dropped by the control thread
//!   ([`Player::collect_garbage`]), so the callback never frees large allocations.
//! * The **loader thread** ([`assets::SampleCache`]) decodes WAV files.
//!
//! When no output device can be opened, a timer thread renders into the void at the device's
//! pace, so the transport, playhead and meters still behave (the UI says audio is unavailable).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod assets;
pub mod devices;

pub use devices::{AudioConfig, DeviceInfo, list_output_devices};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use limeshot_core::Tick;
use limeshot_dsp::SampleData;
use limeshot_engine::{MAX_BLOCK, PlayMode, Renderer};
use limeshot_sequencer::Snapshot;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

const QUEUE: usize = 1024;
const GARBAGE: usize = 256;
const METER_SLOTS: usize = limeshot_core::Mixer::MAX_TRACKS;

/// Control messages for the audio thread.
pub enum Msg {
    Snapshot(Arc<Snapshot>),
    Play,
    Stop,
    Pause,
    Locate(Tick),
    Mode(PlayMode),
    SongLoop(bool),
    /// Song mode loops this range (the Playlist's time selection); `None` loops the whole song.
    LoopRange(Option<(Tick, Tick)>),
    Metronome(bool),
    NoteOn {
        channel: u16,
        key: u8,
        velocity: f32,
    },
    NoteFor {
        channel: u16,
        key: u8,
        velocity: f32,
        seconds: f64,
    },
    NoteOff {
        channel: u16,
        key: u8,
    },
    PreviewSample(Arc<SampleData>),
    /// A processor for insert effect `id`, built on the control thread
    /// (`limeshot_engine::fx::build`). Send it before the snapshot that uses it.
    Effect(u32, Box<dyn limeshot_dsp::effects::Effect>),
    /// The plugin instrument of the channel with this id, built on the control thread. Send it
    /// before the snapshot that uses it.
    Instrument(u32, Box<dyn limeshot_dsp::Instrument>),
    /// Retire all effect processors and plugin instruments (a different project was opened).
    ClearEffects,
    StopPreview,
    Panic,
}

/// Playback state published by the audio thread.
pub struct Status {
    position: AtomicU64,
    playing: AtomicBool,
    song_mode: AtomicBool,
    voices: AtomicU32,
    latency: AtomicU32,
    /// Callback time / buffer duration, in thousandths.
    load: AtomicU32,
    peaks: Vec<AtomicU32>,
    /// Gain reduction per mixer track (dB, as f32 bits).
    reductions: Vec<AtomicU32>,
    clipped: AtomicBool,
    callbacks: AtomicU64,
}

impl Status {
    fn new() -> Self {
        Self {
            position: AtomicU64::new(0),
            playing: AtomicBool::new(false),
            song_mode: AtomicBool::new(false),
            voices: AtomicU32::new(0),
            latency: AtomicU32::new(0),
            load: AtomicU32::new(0),
            peaks: (0..2 * METER_SLOTS).map(|_| AtomicU32::new(0)).collect(),
            reductions: (0..METER_SLOTS).map(|_| AtomicU32::new(0)).collect(),
            clipped: AtomicBool::new(false),
            callbacks: AtomicU64::new(0),
        }
    }

    pub fn position(&self) -> Tick {
        self.position.load(Ordering::Relaxed)
    }

    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    pub fn is_song_mode(&self) -> bool {
        self.song_mode.load(Ordering::Relaxed)
    }

    /// Plugin delay compensation: how far the output lags the transport, in frames.
    pub fn latency(&self) -> u32 {
        self.latency.load(Ordering::Relaxed)
    }

    pub fn voices(&self) -> u32 {
        self.voices.load(Ordering::Relaxed)
    }

    /// Fraction of the buffer period the last callback used (0–1+).
    pub fn load(&self) -> f32 {
        self.load.load(Ordering::Relaxed) as f32 / 1000.0
    }

    /// The (left, right) peak of mixer track `i` in the last callback.
    pub fn peak(&self, i: usize) -> (f32, f32) {
        let g = |j: usize| {
            self.peaks
                .get(j)
                .map_or(0.0, |a| f32::from_bits(a.load(Ordering::Relaxed)))
        };
        (g(2 * i), g(2 * i + 1))
    }

    /// The most gain reduction (dB) of mixer track `i`'s compressors in the last callback.
    pub fn gain_reduction(&self, i: usize) -> f32 {
        self.reductions
            .get(i)
            .map_or(0.0, |a| f32::from_bits(a.load(Ordering::Relaxed)))
    }

    /// Whether the master clipped since the last call.
    pub fn take_clipped(&self) -> bool {
        self.clipped.swap(false, Ordering::Relaxed)
    }

    pub fn callbacks(&self) -> u64 {
        self.callbacks.load(Ordering::Relaxed)
    }
}

/// Objects the audio thread is done with, returned so the control thread frees them.
enum Garbage {
    Snapshot(#[allow(dead_code)] Arc<Snapshot>),
    Effect(#[allow(dead_code)] Box<dyn limeshot_dsp::effects::Effect>),
    Instrument(#[allow(dead_code)] Box<dyn limeshot_dsp::Instrument>),
}

/// What the audio thread owns.
struct AudioState {
    renderer: Renderer,
    rx: rtrb::Consumer<Msg>,
    garbage: rtrb::Producer<Garbage>,
    status: Arc<Status>,
    buf: Vec<f32>,
}

impl AudioState {
    /// Apply up to 256 queued control messages and return retired processors to the control thread.
    /// Retired processors that do not fit in the return queue are kept for a later callback.
    fn drain(&mut self) {
        // Bounded so a flood of messages cannot stall a callback.
        for _ in 0..256 {
            let Ok(msg) = self.rx.pop() else { break };
            let r = &mut self.renderer;
            match msg {
                Msg::Snapshot(s) => {
                    let old = r.set_snapshot(s);
                    // If the return ring is full the old snapshot is dropped here; it only
                    // happens when the control thread stops collecting.
                    let _ = self.garbage.push(Garbage::Snapshot(old));
                }
                Msg::Play => r.play(),
                Msg::Stop => r.stop(),
                Msg::Pause => r.pause(),
                Msg::Locate(t) => r.locate(t),
                Msg::Mode(m) => r.set_mode(m),
                Msg::SongLoop(b) => r.set_song_loop(b),
                Msg::LoopRange(range) => r.set_loop_range(range),
                Msg::Metronome(b) => r.set_metronome(b),
                Msg::NoteOn { channel, key, velocity } => r.preview_note_on(channel, key, velocity),
                Msg::NoteFor {
                    channel,
                    key,
                    velocity,
                    seconds,
                } => r.preview_note_for(channel, key, velocity, seconds),
                Msg::NoteOff { channel, key } => r.preview_note_off(channel, key),
                Msg::PreviewSample(s) => r.preview_sample(s),
                Msg::StopPreview => r.stop_sample_preview(),
                Msg::Panic => r.all_notes_off(),
                Msg::Effect(id, fx) => r.adopt_effect(id, fx),
                Msg::Instrument(channel, inst) => r.adopt_instrument(channel, inst),
                Msg::ClearEffects => r.clear_effects(),
            }
        }
        // Hand retired effect processors back; keep any the ring cannot take for later.
        while let Some(fx) = self.renderer.take_retired_effect() {
            if let Err(rtrb::PushError::Full(Garbage::Effect(fx))) = self.garbage.push(Garbage::Effect(fx)) {
                self.renderer.keep_retired_effect(fx);
                break;
            }
        }
        // Plugins are torn down where they were made (the control thread), never here.
        while let Some(inst) = self.renderer.take_retired_instrument() {
            if let Err(rtrb::PushError::Full(Garbage::Instrument(inst))) = self.garbage.push(Garbage::Instrument(inst))
            {
                self.renderer.keep_retired_instrument(inst);
                break;
            }
        }
    }

    /// Render `frames` frames and hand each stereo chunk to `write(offset, chunk)`.
    fn process(&mut self, frames: usize, mut write: impl FnMut(usize, &[f32])) {
        self.drain();
        let mut done = 0;
        let mut peaks = [(0.0f32, 0.0f32); METER_SLOTS];
        let mut reductions = [0.0f32; METER_SLOTS];
        while done < frames {
            let n = (frames - done).min(MAX_BLOCK);
            let buf = &mut self.buf[..2 * n];
            self.renderer.render(buf);
            write(done, buf);
            for (i, p) in self.renderer.peaks().iter().enumerate().take(METER_SLOTS) {
                peaks[i].0 = peaks[i].0.max(p.left);
                peaks[i].1 = peaks[i].1.max(p.right);
            }
            for (i, r) in self.renderer.gain_reductions().iter().enumerate().take(METER_SLOTS) {
                reductions[i] = reductions[i].max(*r);
            }
            done += n;
        }
        let s = &self.status;
        let tracks = self.renderer.snapshot().mixer.len().min(METER_SLOTS);
        for (i, p) in peaks.iter().enumerate().take(tracks) {
            if let (Some(l), Some(r)) = (s.peaks.get(2 * i), s.peaks.get(2 * i + 1)) {
                l.store(p.0.to_bits(), Ordering::Relaxed);
                r.store(p.1.to_bits(), Ordering::Relaxed);
            }
            if let Some(g) = s.reductions.get(i) {
                g.store(reductions[i].to_bits(), Ordering::Relaxed);
            }
        }
        if self.renderer.take_master_clipped() {
            s.clipped.store(true, Ordering::Relaxed);
        }
        s.position.store(self.renderer.position(), Ordering::Relaxed);
        s.playing.store(self.renderer.is_playing(), Ordering::Relaxed);
        s.song_mode
            .store(self.renderer.mode() == PlayMode::Song, Ordering::Relaxed);
        s.voices.store(self.renderer.active_voices() as u32, Ordering::Relaxed);
        s.latency.store(self.renderer.latency(), Ordering::Relaxed);
        s.callbacks.fetch_add(1, Ordering::Relaxed);
    }
}

fn write_device<T: SizedSample + FromSample<f32>>(data: &mut [T], channels: usize, offset: usize, chunk: &[f32]) {
    for (f, lr) in chunk.as_chunks::<2>().0.iter().enumerate() {
        let base = (offset + f) * channels;
        let Some(frame) = data.get_mut(base..base + channels) else {
            return;
        };
        if channels == 1 {
            frame[0] = T::from_sample((0.5 * (lr[0] + lr[1])).clamp(-1.0, 1.0));
            continue;
        }
        for (c, out) in frame.iter_mut().enumerate() {
            let v = match c {
                0 => lr[0],
                1 => lr[1],
                _ => 0.0,
            };
            *out = T::from_sample(v.clamp(-1.0, 1.0));
        }
    }
}

/// Where the audio is going.
pub enum Output {
    Device {
        _stream: cpal::Stream,
        name: String,
    },
    /// No device: a timer thread keeps the transport running silently.
    Silent {
        reason: String,
        stop: Arc<AtomicBool>,
    },
}

/// The control-thread handle to playback.
pub struct Player {
    tx: rtrb::Producer<Msg>,
    garbage: rtrb::Consumer<Garbage>,
    status: Arc<Status>,
    output: Output,
    sample_rate: u32,
    buffer_frames: Option<u32>,
}

impl Player {
    /// Open the configured output (falling back to the default device, then to silent output)
    /// and start rendering `snapshot`.
    pub fn start(config: &AudioConfig, snapshot: Arc<Snapshot>) -> Self {
        match open_device(config) {
            Ok((device, stream_config, format)) => {
                let sample_rate = stream_config.sample_rate.0;
                let (tx, rx) = rtrb::RingBuffer::new(QUEUE);
                let (gtx, grx) = rtrb::RingBuffer::new(GARBAGE);
                let status = Arc::new(Status::new());
                let state = AudioState {
                    renderer: Renderer::new(sample_rate, snapshot.clone()),
                    rx,
                    garbage: gtx,
                    status: status.clone(),
                    buf: vec![0.0; 2 * MAX_BLOCK],
                };
                let name = device.name().unwrap_or_else(|_| "Output".into());
                let buffer_frames = match stream_config.buffer_size {
                    cpal::BufferSize::Fixed(n) => Some(n),
                    cpal::BufferSize::Default => None,
                };
                match build_stream(&device, &stream_config, format, state) {
                    Ok(stream) => match stream.play() {
                        Ok(()) => {
                            log::info!(
                                "audio: {name} at {sample_rate} Hz, {format:?}, {} ch",
                                stream_config.channels
                            );
                            Self {
                                tx,
                                garbage: grx,
                                status,
                                output: Output::Device { _stream: stream, name },
                                sample_rate,
                                buffer_frames,
                            }
                        }
                        Err(e) => Self::silent(format!("could not start {name}: {e}"), snapshot),
                    },
                    Err(e) => Self::silent(format!("could not open {name}: {e}"), snapshot),
                }
            }
            Err(reason) => Self::silent(reason, snapshot),
        }
    }

    /// Run without a device.
    pub fn silent(reason: String, snapshot: Arc<Snapshot>) -> Self {
        log::warn!("audio output unavailable: {reason}");
        let sample_rate = 48_000;
        let (tx, rx) = rtrb::RingBuffer::new(QUEUE);
        let (gtx, grx) = rtrb::RingBuffer::new(GARBAGE);
        let status = Arc::new(Status::new());
        let mut state = AudioState {
            renderer: Renderer::new(sample_rate, snapshot),
            rx,
            garbage: gtx,
            status: status.clone(),
            buf: vec![0.0; 2 * MAX_BLOCK],
        };
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let spawned = std::thread::Builder::new()
            .name("limeshot-silent-clock".into())
            .spawn(move || {
                let block = 480usize; // 10 ms
                let period = Duration::from_secs_f64(block as f64 / f64::from(sample_rate));
                let mut next = Instant::now();
                while !stop2.load(Ordering::Relaxed) {
                    state.process(block, |_, _| {});
                    next += period;
                    let now = Instant::now();
                    if next > now {
                        std::thread::sleep(next - now);
                    } else {
                        next = now;
                    }
                }
            });
        if let Err(e) = spawned {
            log::error!("could not start the silent clock: {e}");
        }
        Self {
            tx,
            garbage: grx,
            status,
            output: Output::Silent { reason, stop },
            sample_rate,
            buffer_frames: None,
        }
    }

    /// Queue a message. Returns false when the queue is full (the message is dropped).
    pub fn send(&mut self, msg: Msg) -> bool {
        self.tx.push(msg).is_ok()
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn buffer_frames(&self) -> Option<u32> {
        self.buffer_frames
    }

    /// The output device name, or why there is none.
    pub fn output_description(&self) -> Result<&str, &str> {
        match &self.output {
            Output::Device { name, .. } => Ok(name),
            Output::Silent { reason, .. } => Err(reason),
        }
    }

    /// Drop snapshots and processors the audio thread has finished with. Call regularly (every UI
    /// frame) from the thread that created the plugins.
    pub fn collect_garbage(&mut self) {
        while self.garbage.pop().is_ok() {}
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        if let Output::Silent { stop, .. } = &self.output {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

fn open_device(config: &AudioConfig) -> Result<(cpal::Device, cpal::StreamConfig, cpal::SampleFormat), String> {
    let host = cpal::default_host();
    let device = devices::find_output(&host, config.device.as_deref()).ok_or("no audio output device")?;
    let default = device.default_output_config().map_err(|e| e.to_string())?;
    let mut format = default.sample_format();
    let mut sample_rate = default.sample_rate();
    let mut buffer = *default.buffer_size();
    let mut channels = default.channels();
    if let Some(want) = config.sample_rate
        && let Ok(ranges) = device.supported_output_configs()
    {
        for r in ranges {
            if r.min_sample_rate().0 <= want
                && want <= r.max_sample_rate().0
                && devices::supported_format(r.sample_format())
            {
                let c = r.with_sample_rate(cpal::SampleRate(want));
                format = c.sample_format();
                sample_rate = c.sample_rate();
                buffer = *c.buffer_size();
                channels = c.channels();
                break;
            }
        }
    }
    if !devices::supported_format(format) {
        format = cpal::SampleFormat::F32;
    }
    let stream_config = cpal::StreamConfig {
        channels,
        sample_rate,
        buffer_size: devices::buffer_size(config.buffer_size, &buffer),
    };
    Ok((device, stream_config, format))
}

fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    format: cpal::SampleFormat,
    state: AudioState,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    match format {
        cpal::SampleFormat::I16 => build_typed::<i16>(device, config, state),
        cpal::SampleFormat::I32 => build_typed::<i32>(device, config, state),
        cpal::SampleFormat::U16 => build_typed::<u16>(device, config, state),
        _ => build_typed::<f32>(device, config, state),
    }
}

fn build_typed<T: SizedSample + FromSample<f32> + Send + 'static>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut state: AudioState,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let channels = usize::from(config.channels.max(1));
    let rate = f64::from(config.sample_rate.0.max(1));
    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            let t0 = Instant::now();
            let frames = data.len() / channels;
            state.process(frames, |off, chunk| write_device(data, channels, off, chunk));
            let budget = frames as f64 / rate;
            if budget > 0.0 {
                let load = (t0.elapsed().as_secs_f64() / budget * 1000.0) as u32;
                state.status.load.store(load, Ordering::Relaxed);
            }
        },
        |e| log::error!("audio stream error: {e}"),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silent_player_runs_the_transport() {
        let mut p = Player::silent("test".into(), Arc::new(Snapshot::default()));
        assert!(p.send(Msg::Play));
        let t0 = Instant::now();
        while p.status().position() == 0 && t0.elapsed() < Duration::from_secs(3) {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(p.status().is_playing());
        assert!(p.status().position() > 0);
        assert!(p.send(Msg::Snapshot(Arc::new(Snapshot::default()))));
        std::thread::sleep(Duration::from_millis(50));
        p.collect_garbage();
        assert!(p.status().callbacks() > 0);
    }

    #[test]
    fn device_writer_handles_mono_and_wide_devices() {
        let chunk = [0.5f32, -0.5, 1.5, 1.5];
        let mut mono = [0.0f32; 2];
        write_device(&mut mono, 1, 0, &chunk);
        assert_eq!(mono, [0.0, 1.0]);
        let mut quad = [9.0f32; 8];
        write_device(&mut quad, 4, 0, &chunk);
        assert_eq!(quad, [0.5, -0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]);
        let mut short = [0i16; 2];
        write_device(&mut short, 2, 0, &chunk); // second frame does not fit: ignored
        assert_eq!(
            short,
            [
                <i16 as FromSample<f32>>::from_sample_(0.5),
                <i16 as FromSample<f32>>::from_sample_(-0.5)
            ]
        );
    }
}
