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
        let (producer, consumer) = rtrb::RingBuffer::new(2 * rate as usize * RING_SECONDS);
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            level: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
        });
        let writer = spawn_writer(consumer, path, rate, shared.clone())?;
        let stream = match format {
            cpal::SampleFormat::I16 => build::<i16>(&dev, &config, producer, shared.clone()),
            cpal::SampleFormat::I32 => build::<i32>(&dev, &config, producer, shared.clone()),
            cpal::SampleFormat::U16 => build::<u16>(&dev, &config, producer, shared.clone()),
            _ => build::<f32>(&dev, &config, producer, shared.clone()),
        }
        .map_err(|e| e.to_string());
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                shared.stop.store(true, Ordering::Release);
                let _ = writer.join();
                return Err(e);
            }
        };
        if let Err(e) = stream.play() {
            shared.stop.store(true, Ordering::Release);
            let _ = writer.join();
            return Err(e.to_string());
        }
        Ok(InputRecorder {
            _stream: Some(stream),
            shared,
            writer: Some(writer),
            sample_rate: rate,
            device: name,
        })
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

fn build<T: SizedSample + Send + 'static>(
    dev: &cpal::Device,
    config: &cpal::StreamConfig,
    mut producer: rtrb::Producer<f32>,
    shared: Arc<Shared>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels.max(1));
    dev.build_input_stream(
        config,
        move |data: &[T], _| {
            // Real-time: no allocation, no locks; frames that do not fit are counted, not kept.
            let mut peak = 0.0f32;
            let mut dropped = 0u32;
            for frame in data.chunks(channels) {
                let l: f32 = frame.first().map_or(0.0, |s| s.to_sample());
                let r = if channels > 1 {
                    frame.get(1).map_or(l, |s| s.to_sample())
                } else {
                    l
                };
                peak = peak.max(l.abs()).max(r.abs());
                if producer.slots() >= 2 {
                    let _ = producer.push(l);
                    let _ = producer.push(r);
                } else {
                    dropped += 1;
                }
            }
            shared.level.store(peak.to_bits(), Ordering::Relaxed);
            if dropped > 0 {
                shared.dropped.fetch_add(dropped, Ordering::Relaxed);
            }
        },
        |e| log::error!("audio input error: {e}"),
        None,
    )
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
}
