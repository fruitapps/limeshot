//! Bounded, lock-free sample reads. File access and page replacement belong to a worker.
use crate::{SampleData, SampleError};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Duration,
};

/// Maximum decoded stereo float size, independent of the source compression or sample rate.
pub const MAX_DISK_BYTES: usize = 2_000_000_000;
pub const MAX_DISK_FRAMES: usize = MAX_DISK_BYTES / 8;
pub const PEAK_BLOCK: usize = 1024;
const PAGE: usize = 4096;
const SLOTS: usize = 32;
const EMPTY: u64 = u64::MAX;

struct Page {
    sequence: AtomicU64,
    tag: AtomicU64,
    request: AtomicU64,
    data: Box<[AtomicU32]>,
}
struct Shared {
    pages: Vec<Page>,
    failed: AtomicBool,
    offline: AtomicBool,
    misses: AtomicU64,
}

pub struct DiskSample {
    shared: Arc<Shared>,
    pub frames: usize,
    pub peaks: Vec<(f32, f32)>,
    pub cache: PathBuf,
}
impl std::fmt::Debug for DiskSample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskSample")
            .field("frames", &self.frames)
            .field("cache", &self.cache)
            .finish()
    }
}
impl PartialEq for DiskSample {
    fn eq(&self, other: &Self) -> bool {
        self.cache == other.cache
    }
}
impl DiskSample {
    pub fn set_offline(&self, offline: bool) {
        self.shared.offline.store(offline, Ordering::Relaxed);
    }
    pub fn failed(&self) -> bool {
        self.shared.failed.load(Ordering::Relaxed)
    }
    pub fn underruns(&self) -> u64 {
        self.shared.misses.load(Ordering::Relaxed)
    }
    fn request(&self, page: usize) {
        if page * PAGE >= self.frames {
            return;
        }
        let slot = &self.shared.pages[page % SLOTS];
        if slot.tag.load(Ordering::Acquire) != page as u64 {
            slot.request.store(page as u64, Ordering::Release);
        }
    }
    /// Read only atomics in realtime. An unavailable page yields silence rather than blocking.
    pub fn frame(&self, frame: usize) -> (f32, f32) {
        if frame >= self.frames {
            return (0.0, 0.0);
        }
        let page = frame / PAGE;
        let slot = &self.shared.pages[page % SLOTS];
        let index = (frame % PAGE) * 2;
        loop {
            let sequence = slot.sequence.load(Ordering::Acquire);
            if sequence.is_multiple_of(2) && slot.tag.load(Ordering::Acquire) == page as u64 {
                let left = f32::from_bits(slot.data[index].load(Ordering::Relaxed));
                let right = f32::from_bits(slot.data[index + 1].load(Ordering::Relaxed));
                std::sync::atomic::fence(Ordering::Acquire);
                if slot.sequence.load(Ordering::Acquire) == sequence {
                    // Fractional/pitched playback may skip exact page midpoints.
                    self.request(page + 1);
                    self.request(page + 2);
                    if page > 0 {
                        self.request(page - 1);
                    }
                    return (left, right);
                }
            }
            self.request(page);
            if !self.shared.offline.load(Ordering::Relaxed) || self.shared.failed.load(Ordering::Relaxed) {
                self.shared.misses.fetch_add(1, Ordering::Relaxed);
                return (0.0, 0.0);
            }
            // Offline renders may wait for disk; the realtime path above never waits.
            std::thread::sleep(Duration::from_micros(50));
        }
    }
}
fn fill(
    reader: &mut hound::WavReader<std::io::BufReader<std::fs::File>>,
    slot: &Page,
    page: u64,
    frames: usize,
) -> Result<(), hound::Error> {
    reader.seek((page as usize * PAGE) as u32)?;
    slot.sequence.fetch_add(1, Ordering::AcqRel);
    slot.tag.store(EMPTY, Ordering::Release);
    let count = PAGE.min(frames.saturating_sub(page as usize * PAGE)) * 2;
    let mut samples = reader.samples::<f32>();
    for (i, target) in slot.data.iter().enumerate() {
        let value = if i < count {
            samples.next().transpose()?.unwrap_or(0.0)
        } else {
            0.0
        };
        target.store(value.to_bits(), Ordering::Relaxed);
    }
    slot.tag.store(page, Ordering::Release);
    slot.sequence.fetch_add(1, Ordering::Release);
    Ok(())
}
fn error(path: &Path, e: impl std::fmt::Display) -> SampleError {
    SampleError::Decode {
        path: path.to_path_buf(),
        message: e.to_string(),
    }
}

/// Decode a seekable float cache and extrema in a worker/control context. Cache identity includes
/// canonical source path, size and modification time. A failed preparation never publishes a cache.
pub fn load_disk(path: &Path) -> Result<SampleData, SampleError> {
    use std::hash::{Hash, Hasher};
    let source = path.canonicalize().map_err(|e| error(path, e))?;
    let metadata = source.metadata().map_err(|e| error(path, e))?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hash);
    metadata.len().hash(&mut hash);
    metadata.modified().ok().hash(&mut hash);
    let root = std::env::temp_dir().join("limeshot-audio-cache-v1");
    std::fs::create_dir_all(&root).map_err(|e| error(path, e))?;
    let cache = root.join(format!("{:016x}.wav", hash.finish()));
    let peak_file = cache.with_extension("peaks");
    static PREPARATION: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = PREPARATION
        .lock()
        .map_err(|_| error(path, "Disk cache preparation was interrupted"))?;
    if !cache.is_file() || !peak_file.is_file() {
        let unique = format!(
            "{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let temporary = cache.with_extension(format!("{unique}.tmp"));
        let peak_tmp = cache.with_extension(format!("{unique}.peaks.tmp"));
        let result = prepare(&source, &temporary, &peak_tmp);
        if let Err(e) = result {
            let _ = std::fs::remove_file(temporary);
            let _ = std::fs::remove_file(peak_tmp);
            return Err(e);
        }
        std::fs::rename(&temporary, &cache).map_err(|e| error(path, e))?;
        std::fs::rename(&peak_tmp, &peak_file).map_err(|e| error(path, e))?;
    }
    drop(guard);
    let mut reader = hound::WavReader::open(&cache).map_err(|e| error(path, e))?;
    let frames = reader.duration() as usize;
    if !(2..=MAX_DISK_FRAMES).contains(&frames)
        || reader.spec().channels != 2
        || reader.spec().sample_format != hound::SampleFormat::Float
    {
        return Err(error(path, "Invalid disk cache"));
    }
    let rate = reader.spec().sample_rate;
    let expected = frames.div_ceil(PEAK_BLOCK) * 8;
    if peak_file.metadata().map_err(|e| error(path, e))?.len() != expected as u64 {
        return Err(error(path, "Invalid peak cache"));
    }
    let mut bytes = Vec::with_capacity(expected);
    std::fs::File::open(&peak_file)
        .map_err(|e| error(path, e))?
        .take(expected as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| error(path, e))?;
    let peaks = bytes
        .as_chunks::<8>()
        .0
        .iter()
        .map(|b| {
            (
                f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                f32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            )
        })
        .collect();
    let shared = Arc::new(Shared {
        pages: (0..SLOTS)
            .map(|_| Page {
                sequence: AtomicU64::new(0),
                tag: AtomicU64::new(EMPTY),
                request: AtomicU64::new(EMPTY),
                data: (0..PAGE * 2).map(|_| AtomicU32::new(0)).collect(),
            })
            .collect(),
        failed: AtomicBool::new(false),
        offline: AtomicBool::new(false),
        misses: AtomicU64::new(0),
    });
    for page in 0..4.min(frames.div_ceil(PAGE)) {
        fill(&mut reader, &shared.pages[page], page as u64, frames).map_err(|e| error(path, e))?;
    }
    let weak = Arc::downgrade(&shared);
    std::thread::Builder::new()
        .name("limeshot-disk-read".into())
        .spawn(move || {
            while let Some(shared) = weak.upgrade() {
                for slot in &shared.pages {
                    let page = slot.request.swap(EMPTY, Ordering::AcqRel);
                    if page != EMPTY
                        && slot.tag.load(Ordering::Acquire) != page
                        && fill(&mut reader, slot, page, frames).is_err()
                    {
                        shared.failed.store(true, Ordering::Relaxed);
                        return;
                    }
                }
                drop(shared);
                std::thread::sleep(Duration::from_millis(2));
            }
        })
        .map_err(|e| error(path, e))?;
    Ok(SampleData {
        sample_rate: rate,
        data: Vec::new(),
        source: Some(source),
        disk: Some(Arc::new(DiskSample {
            shared,
            frames,
            peaks,
            cache,
        })),
    })
}
fn prepare(source: &Path, target: &Path, peak_file: &Path) -> Result<(), SampleError> {
    // Symphonia decodes packets, never the complete file. The output cache is always seekable WAV.
    let mut writer = None;
    let mut peaks = Vec::new();
    let (mut lo, mut hi) = (0.0f32, 0.0f32);
    let mut count = 0usize;
    crate::sample::decode_to(source, MAX_DISK_FRAMES, |rate, left, right| {
        if writer.is_none() {
            writer = Some(
                hound::WavWriter::create(
                    target,
                    hound::WavSpec {
                        channels: 2,
                        sample_rate: rate,
                        bits_per_sample: 32,
                        sample_format: hound::SampleFormat::Float,
                    },
                )
                .map_err(|e| error(source, e))?,
            );
        }
        if let Some(writer) = &mut writer {
            writer
                .write_sample(left)
                .and_then(|_| writer.write_sample(right))
                .map_err(|e| error(source, e))?;
        }
        let mono = (left + right) * 0.5;
        lo = lo.min(mono);
        hi = hi.max(mono);
        count += 1;
        if count.is_multiple_of(PEAK_BLOCK) {
            peaks.push((lo, hi));
            lo = 0.0;
            hi = 0.0;
        }
        Ok(())
    })?;
    if !count.is_multiple_of(PEAK_BLOCK) {
        peaks.push((lo, hi));
    }
    if let Some(writer) = writer {
        writer.finalize().map_err(|e| error(source, e))?;
    }
    let mut out = std::io::BufWriter::new(std::fs::File::create(peak_file).map_err(|e| error(source, e))?);
    for (lo, hi) in peaks {
        out.write_all(&lo.to_le_bytes())
            .and_then(|_| out.write_all(&hi.to_le_bytes()))
            .map_err(|e| error(source, e))?;
    }
    out.flush().map_err(|e| error(source, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_playback_reads_request_ahead_without_exact_midpoints() {
        let shared = Arc::new(Shared {
            pages: (0..SLOTS)
                .map(|page| Page {
                    sequence: AtomicU64::new(0),
                    tag: AtomicU64::new(if page == 0 { 0 } else { EMPTY }),
                    request: AtomicU64::new(EMPTY),
                    data: (0..PAGE * 2).map(|_| AtomicU32::new(0)).collect(),
                })
                .collect(),
            failed: AtomicBool::new(false),
            offline: AtomicBool::new(false),
            misses: AtomicU64::new(0),
        });
        let disk = DiskSample {
            shared,
            frames: 10 * PAGE,
            peaks: Vec::new(),
            cache: PathBuf::new(),
        };
        // +22 semitones reads positions spaced about 3.56 frames apart.
        let position = 575.0 * 2.0_f64.powf(22.0 / 12.0);
        assert!(!(position as usize).is_multiple_of(PAGE / 2));
        disk.frame(position as usize);
        assert_eq!(disk.shared.pages[1].request.load(Ordering::Acquire), 1);
        assert_eq!(disk.shared.pages[2].request.load(Ordering::Acquire), 2);
        assert_eq!(disk.underruns(), 0);
    }

    #[test]
    fn disk_cache_is_bounded_and_seeks_match_pcm() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.wav");
        let frames = 180_000;
        let data: Vec<f32> = (0..frames * 2).map(|i| ((i % 997) as f32 / 500.0) - 1.0).collect();
        crate::write_wav(&source, 48_000, &data, crate::BitDepth::Float32).unwrap();
        let sample = load_disk(&source).unwrap();
        assert!(sample.data.is_empty());
        assert_eq!(sample.frames(), frames);
        let disk = sample.disk.as_ref().unwrap();
        assert_eq!(disk.shared.pages.len() * PAGE * 2 * 4, 1_048_576);
        disk.set_offline(true);
        for frame in [0, PAGE - 1, PAGE, 160_000, 17, frames - 1] {
            assert_eq!(
                sample.frame(frame),
                (data[frame * 2], data[frame * 2 + 1]),
                "frame {frame}"
            );
        }
        assert_eq!(sample.frame(frames), (0.0, 0.0));
        assert!(sample.overview_range(0, frames - 1).unwrap().1 > 0.9);
        let second = load_disk(&source).unwrap();
        assert_eq!(disk.cache, second.disk.as_ref().unwrap().cache);
    }
    #[test]
    fn declared_audio_over_two_gb_is_rejected_without_decoding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("oversized.wav");
        let length = (MAX_DISK_FRAMES as u32 + 1) * 8;
        let mut header = Vec::new();
        header.extend(b"RIFF");
        header.extend((length + 36).to_le_bytes());
        header.extend(b"WAVEfmt ");
        header.extend(16u32.to_le_bytes());
        header.extend(3u16.to_le_bytes());
        header.extend(2u16.to_le_bytes());
        header.extend(48_000u32.to_le_bytes());
        header.extend(384_000u32.to_le_bytes());
        header.extend(8u16.to_le_bytes());
        header.extend(32u16.to_le_bytes());
        header.extend(b"data");
        header.extend(length.to_le_bytes());
        std::fs::write(&path, header).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(u64::from(length) + 44)
            .unwrap();
        assert!(matches!(load_disk(&path), Err(SampleError::TooLong { .. })));
    }
}
