//! Audio recording from an input device (FL Studio's Record with the Audio filter): the input
//! callback copies frames (as stereo f32) into a lock-free ring, and a writer thread drains it
//! into a 32-bit float WAV file. Stopping finishes the file and reports the take. The input level
//! is kept for a meter.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::JoinHandle;

/// Seconds of audio the ring holds while the writer catches up.
const RING_SECONDS: usize = 4;

/// A finished recording.
#[derive(Debug, Clone, PartialEq)]
pub struct Take {
    pub path: PathBuf,
    pub frames: u64,
    pub sample_rate: u32,
    /// Frames the writer could not keep up with (dropped).
    pub dropped: u64,
}

/// The names of the input devices of the default host.
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|d| d.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}

/// What the input callback and the writer share.
struct Shared {
    stop: AtomicBool,
    /// The last block's peak (f32 bits), for the input meter.
    level: AtomicU32,
    dropped: AtomicU32,
    failed: AtomicBool,
}

/// A running recording.
pub struct InputRecorder {
    _stream: Option<cpal::Stream>,
    shared: Arc<Shared>,
    writer: Option<JoinHandle<Result<Take, String>>>,
    pub sample_rate: u32,
    pub device: String,
}

impl InputRecorder {
    /// Start recording from the input device called `device` (`None`: the default input) into
    /// `path`.
    pub fn start(device: Option<&str>, path: PathBuf) -> Result<Self, String> {
        Self::open(device, Some(path), None, |_| None)
    }

    /// A single input stream feeds both the recording writer and optional live monitoring.
    pub fn start_monitored(
        device: Option<&str>,
        path: PathBuf,
        gate: Option<Arc<AtomicBool>>,
        monitor: impl FnOnce(u32) -> Option<crate::monitor::Sender>,
    ) -> Result<Self, String> {
        Self::open(device, Some(path), gate, monitor)
    }

    /// Monitor without creating a recording file.
    pub fn monitor(
        device: Option<&str>,
        monitor: impl FnOnce(u32) -> Option<crate::monitor::Sender>,
    ) -> Result<Self, String> {
        Self::open(device, None, None, monitor)
    }

    fn open(
        device: Option<&str>,
        path: Option<PathBuf>,
        gate: Option<Arc<AtomicBool>>,
        monitor: impl FnOnce(u32) -> Option<crate::monitor::Sender>,
    ) -> Result<Self, String> {
        let host = cpal::default_host();
        let dev = match device {
            Some(name) => host
                .input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().is_ok_and(|n| n == name))
                .ok_or_else(|| format!("audio input \"{name}\" is not connected"))?,
            None => host.default_input_device().ok_or("no audio input device")?,
        };
        let name = dev.name().unwrap_or_default();
        let supported = dev.default_input_config().map_err(|e| e.to_string())?;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let rate = config.sample_rate.0.max(1);
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            level: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            failed: AtomicBool::new(false),
        });
        let (producer, writer) = if let Some(path) = path {
            let (producer, consumer) = rtrb::RingBuffer::new(2 * rate as usize * RING_SECONDS);
            let writer = spawn_writer(consumer, path, rate, shared.clone())?;
            (Some(producer), Some(writer))
        } else {
            (None, None)
        };
        let monitor = monitor(rate);
        let stream = match format {
            cpal::SampleFormat::I16 => build::<i16>(&dev, &config, producer, monitor, gate, shared.clone()),
            cpal::SampleFormat::I32 => build::<i32>(&dev, &config, producer, monitor, gate, shared.clone()),
            cpal::SampleFormat::U16 => build::<u16>(&dev, &config, producer, monitor, gate, shared.clone()),
            cpal::SampleFormat::F32 => build::<f32>(&dev, &config, producer, monitor, gate, shared.clone()),
            cpal::SampleFormat::F64 => build::<f64>(&dev, &config, producer, monitor, gate, shared.clone()),
            _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
        }
        .map_err(|e| e.to_string());
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                shared.stop.store(true, Ordering::Release);
                if let Some(w) = writer {
                    let _ = w.join();
                }
                return Err(e);
            }
        };
        if let Err(e) = stream.play() {
            shared.stop.store(true, Ordering::Release);
            if let Some(w) = writer {
                let _ = w.join();
            }
            return Err(e.to_string());
        }
        Ok(InputRecorder {
            _stream: Some(stream),
            shared,
            writer,
            sample_rate: rate,
            device: name,
        })
    }

    pub fn failed(&self) -> bool {
        self.shared.failed.load(Ordering::Relaxed)
    }

    /// The input's recent peak level (linear).
    pub fn level(&self) -> f32 {
        f32::from_bits(self.shared.level.load(Ordering::Relaxed))
    }

    /// Stop and finish the file.
    pub fn stop(mut self) -> Result<Take, String> {
        // Dropping the stream stops the callback before the writer drains the rest.
        self._stream = None;
        self.shared.stop.store(true, Ordering::Release);
        self.writer
            .take()
            .ok_or("not recording")?
            .join()
            .map_err(|_| "the recording writer stopped unexpectedly".to_string())?
    }
}

impl Drop for InputRecorder {
    fn drop(&mut self) {
        self._stream = None;
        self.shared.stop.store(true, Ordering::Release);
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}

fn build<T: SizedSample + Send + 'static>(
    dev: &cpal::Device,
    config: &cpal::StreamConfig,
    mut producer: Option<rtrb::Producer<f32>>,
    mut monitor: Option<crate::monitor::Sender>,
    gate: Option<Arc<AtomicBool>>,
    shared: Arc<Shared>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels.max(1));
    let errors = shared.clone();
    dev.build_input_stream(
        config,
        move |data: &[T], _| {
            // Real-time: no allocation, no locks; frames that do not fit are counted, not kept.
            let record = gate.as_ref().is_none_or(|g| g.load(Ordering::Acquire));
            copy_input(data, channels, &mut producer, &mut monitor, record, &shared);
        },
        move |_| errors.failed.store(true, Ordering::Relaxed),
        None,
    )
}

fn copy_input<T: SizedSample>(
    data: &[T],
    channels: usize,
    producer: &mut Option<rtrb::Producer<f32>>,
    monitor: &mut Option<crate::monitor::Sender>,
    record: bool,
    shared: &Shared,
) where
    f32: FromSample<T>,
{
    let mut peak = 0.0f32;
    let mut dropped = 0u32;
    for frame in data.chunks(channels) {
        let l: f32 = frame.first().map_or(0.0, |s| s.to_sample());
        let r = if channels > 1 {
            frame.get(1).map_or(l, |s| s.to_sample())
        } else {
            l
        };
        let l = if l.is_finite() { l } else { 0.0 };
        let r = if r.is_finite() { r } else { 0.0 };
        peak = peak.max(l.abs()).max(r.abs());
        if let Some(m) = monitor {
            m.push([l, r]);
        }
        if record && let Some(p) = producer {
            if p.slots() >= 2 {
                let _ = p.push(l);
                let _ = p.push(r);
            } else {
                dropped = dropped.saturating_add(1);
            }
        }
    }
    shared.level.store(peak.to_bits(), Ordering::Relaxed);
    if dropped > 0 {
        shared.dropped.fetch_add(dropped, Ordering::Relaxed);
    }
}

/// The writer thread: drains `consumer` into a stereo 32-bit float WAV at `path` until
/// `shared.stop` is set and the ring is empty.
fn spawn_writer(
    mut consumer: rtrb::Consumer<f32>,
    path: PathBuf,
    sample_rate: u32,
    shared: Arc<Shared>,
) -> Result<JoinHandle<Result<Take, String>>, String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut wav = hound::WavWriter::create(&path, spec).map_err(|e| format!("{}: {e}", path.display()))?;
    std::thread::Builder::new()
        .name("limeshot-recorder".into())
        .spawn(move || {
            let mut samples = 0u64;
            loop {
                let stopping = shared.stop.load(Ordering::Acquire);
                let n = consumer.slots();
                if n > 0 {
                    let chunk = consumer.read_chunk(n).map_err(|e| e.to_string())?;
                    for s in chunk.into_iter() {
                        wav.write_sample(s).map_err(|e| e.to_string())?;
                        samples += 1;
                    }
                } else if stopping {
                    break;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            wav.finalize().map_err(|e| e.to_string())?;
            Ok(Take {
                path,
                frames: samples / 2,
                sample_rate,
                dropped: u64::from(shared.dropped.load(Ordering::Relaxed)),
            })
        })
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The writer half on its own: what goes into the ring comes out in the WAV, also what is
    /// still queued when recording stops.
    #[test]
    fn the_writer_drains_the_ring_into_a_wav() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Recorded/take 1.wav");
        let (mut p, c) = rtrb::RingBuffer::new(1024);
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            level: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            failed: AtomicBool::new(false),
        });
        let writer = spawn_writer(c, path.clone(), 48_000, shared.clone()).unwrap();
        let mut sent = Vec::new();
        for i in 0..3000 {
            let v = (i as f32 / 3000.0) * if i % 2 == 0 { 1.0 } else { -1.0 };
            while p.push(v).is_err() {
                std::thread::yield_now();
            }
            sent.push(v);
        }
        shared.stop.store(true, Ordering::Release);
        let take = writer.join().unwrap().unwrap();
        assert_eq!((take.frames, take.sample_rate, take.dropped), (1500, 48_000, 0));
        let back = limeshot_dsp::load_wav(&path).unwrap();
        assert_eq!(back.data, sent);
    }
    #[test]
    fn count_in_gate_only_blocks_recording_and_sanitizes_input() {
        let shared = Shared {
            stop: AtomicBool::new(false),
            level: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            failed: AtomicBool::new(false),
        };
        let (tx, mut rx) = rtrb::RingBuffer::new(4);
        let mut writer = Some(tx);
        let (sender, mut receiver) = crate::monitor::channel(48_000, 48_000);
        let control = sender.control();
        control.set_track(Some(1));
        let mut monitor = Some(sender);
        copy_input(&[0.25f32, f32::NAN], 1, &mut writer, &mut monitor, false, &shared);
        assert_eq!(rx.slots(), 0, "count-in must not enter the WAV");
        let mut out = [0.0; 4];
        assert_eq!(receiver.read(&mut out), Some(1));
        assert_eq!(out, [0.25, 0.25, 0.0, 0.0], "monitoring continues during count-in");
        copy_input(&[0.5f32, -0.5, 123.0], 3, &mut writer, &mut monitor, true, &shared);
        copy_input(
            &[f32::INFINITY, f32::NEG_INFINITY],
            2,
            &mut writer,
            &mut monitor,
            true,
            &shared,
        );
        copy_input(&[1.0f32, 1.0], 2, &mut writer, &mut monitor, true, &shared);
        assert_eq!(
            shared.dropped.load(Ordering::Relaxed),
            1,
            "drop whole frames at capacity"
        );
        assert_eq!(
            (
                rx.pop().unwrap(),
                rx.pop().unwrap(),
                rx.pop().unwrap(),
                rx.pop().unwrap()
            ),
            (0.5, -0.5, 0.0, 0.0)
        );
        assert_eq!(f32::from_bits(shared.level.load(Ordering::Relaxed)), 1.0);
    }
}
