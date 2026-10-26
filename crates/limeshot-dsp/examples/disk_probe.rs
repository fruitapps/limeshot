//! Inspect preparation time, read-ahead memory and offline seeking for a sample.
//! Run under /usr/bin/time -l to record peak process RSS as well.
use std::{error::Error, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let source = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("Usage: disk_probe <audio file>")?;
    let start = Instant::now();
    let sample = limeshot_dsp::disk::load_disk(&source)?;
    let prepared = start.elapsed();
    let disk = sample.disk.as_ref().ok_or("disk reader missing")?;
    disk.set_offline(true);
    let start = Instant::now();
    let frames = (sample.sample_rate as usize * 5).min(sample.frames());
    let first = sample.frames().saturating_sub(frames) / 2;
    let checksum = (first..first + frames).fold(0.0f64, |sum, frame| {
        let (left, right) = sample.frame(frame);
        sum + f64::from(left) + f64::from(right)
    });
    println!(
        "duration_seconds={:.3} sample_rate={} frames={} prepare_ms={:.2}",
        sample.duration_secs(),
        sample.sample_rate,
        sample.frames(),
        prepared.as_secs_f64() * 1000.0
    );
    println!(
        "embedded_pcm_bytes={} page_audio_bytes=1048576 peak_bytes={} seek_read_ms={:.2} checksum={checksum:.6}",
        sample.data.len() * 4,
        disk.peaks.len() * 8,
        start.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}
