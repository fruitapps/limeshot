//! Fixed-duration stereo-linked WSOLA preparation. Control/worker threads only.
//! No streaming state or tempo-following behavior is implied by this API.
use crate::SampleData;
use std::sync::atomic::{AtomicBool, Ordering};

/// Bound every retained source and result, and every intermediate buffer (48 MB each).
pub const MAX_PREPARED_FRAMES: usize = 3_000_000;

#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error("Use 2–3,000,000 stereo frames at 8–384 kHz")]
    Source,
    #[error("Duration must be 0.5–2×, pitch −12…12 st, and output at most 3,000,000 frames")]
    Range,
    #[error("Preparation cancelled")]
    Cancelled,
}

fn cancelled(cancel: &AtomicBool) -> Result<(), PrepareError> {
    if cancel.load(Ordering::Relaxed) {
        Err(PrepareError::Cancelled)
    } else {
        Ok(())
    }
}

/// Resample or independently stretch duration and pitch. `ratio` is output/source duration.
/// Resample couples pitch to duration and requires zero independent pitch. Stretch preserves
/// pitch at the root key; ordinary keyboard transposition remains resampling afterwards.
/// Finite PCM is required. Sources shorter than a grain use resampling (documented in the UI).
/// Original and prepared regions map by their normalized frame position, not transient identity.
pub fn prepare(
    source: &SampleData,
    ratio: f64,
    pitch: f64,
    stretch: bool,
    cancel: &AtomicBool,
) -> Result<SampleData, PrepareError> {
    let n = source.frames();
    if source.disk.is_some()
        || !(2..=MAX_PREPARED_FRAMES).contains(&n)
        || !(8_000..=384_000).contains(&source.sample_rate)
        || source.data.len() != n * 2
        || source.data.iter().any(|v| !v.is_finite())
    {
        return Err(PrepareError::Source);
    }
    let frames = (n as f64 * ratio).round() as usize;
    if !(0.5..=2.0).contains(&ratio)
        || !(-12.0..=12.0).contains(&pitch)
        || (!stretch && pitch != 0.0)
        || !(2..=MAX_PREPARED_FRAMES).contains(&frames)
    {
        return Err(PrepareError::Range);
    }
    cancelled(cancel)?;
    if !stretch {
        return Ok(SampleData::from_stereo(
            source.sample_rate,
            resample(&source.data, frames, cancel)?,
        ));
    }
    let pitch_frames = (n as f64 / 2f64.powf(pitch / 12.0)).round() as usize;
    if !(2..=MAX_PREPARED_FRAMES).contains(&pitch_frames) {
        return Err(PrepareError::Range);
    }
    let input = resample(&source.data, pitch_frames, cancel)?;
    let hop = (source.sample_rate as usize / 100).max(16); // 20 ms grains, 10 ms overlap
    if input.len() / 2 < hop * 2 || frames < hop * 2 {
        return Ok(SampleData::from_stereo(
            source.sample_rate,
            resample(&input, frames, cancel)?,
        ));
    }
    if pitch_frames == frames {
        return Ok(SampleData::from_stereo(source.sample_rate, input));
    }
    let mut output = vec![0f32; frames * 2];
    let mut weights = vec![0f32; frames];
    let window: Vec<f32> = (0..hop * 2)
        .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (hop * 2) as f32).cos())
        .collect();
    let search = hop / 2;
    let mut previous = 0usize;
    // Center the first grain at zero: normalization preserves the onset rather than fading it.
    for center in (0..frames + hop).step_by(hop) {
        cancelled(cancel)?;
        let expected = ((center as f64 * pitch_frames as f64 / frames as f64).round() as usize).min(pitch_frames - 1);
        let mut best = expected;
        if center > 0 && expected + hop < pitch_frames {
            let target = previous + hop;
            let lo = expected.saturating_sub(search);
            let hi = (expected + search).min(pitch_frames - hop);
            let stride = (hop / 64).max(1);
            let score = |candidate: usize| {
                let mut dot = 0f64;
                let mut aa = 0f64;
                let mut bb = 0f64;
                for i in (0..hop).step_by(stride) {
                    let a = target.saturating_sub(hop) + i;
                    let b = candidate.saturating_sub(hop) + i;
                    if a >= pitch_frames || b >= pitch_frames {
                        continue;
                    }
                    for ch in 0..2 {
                        let x = f64::from(input[2 * a + ch]);
                        let y = f64::from(input[2 * b + ch]);
                        dot += x * y;
                        aa += x * x;
                        bb += y * y;
                    }
                }
                dot / (aa * bb).sqrt().max(1e-20) - candidate.abs_diff(expected) as f64 * 1e-9
            };
            let coarse = (source.sample_rate as usize / 12_000).max(1);
            let mut best_score = score(best);
            for candidate in (lo..=hi).step_by(coarse) {
                let value = score(candidate);
                if value > best_score {
                    best_score = value;
                    best = candidate;
                }
            }
            let near = best;
            for candidate in near.saturating_sub(coarse).max(lo)..=(near + coarse).min(hi) {
                let value = score(candidate);
                if value > best_score {
                    best_score = value;
                    best = candidate;
                }
            }
        }
        previous = best;
        for (j, &w) in window.iter().enumerate() {
            let dst = center as i64 + j as i64 - hop as i64;
            let src = best as i64 + j as i64 - hop as i64;
            if dst < 0 || dst >= frames as i64 || src < 0 || src >= pitch_frames as i64 {
                continue;
            }
            let dst = dst as usize;
            let src = src as usize;
            output[2 * dst] += input[2 * src] * w;
            output[2 * dst + 1] += input[2 * src + 1] * w;
            weights[dst] += w;
        }
    }
    for (frame, weight) in output.as_chunks_mut::<2>().0.iter_mut().zip(weights) {
        if weight > 1e-9 {
            frame[0] /= weight;
            frame[1] /= weight;
        }
    }
    Ok(SampleData::from_stereo(source.sample_rate, output))
}

// Windowed-sinc interpolation uses one kernel for both channels. All scratch is worker-owned.
fn resample(input: &[f32], frames: usize, cancel: &AtomicBool) -> Result<Vec<f32>, PrepareError> {
    let n = input.len() / 2;
    if n == frames {
        return Ok(input.to_vec());
    }
    let step = n as f64 / frames as f64;
    let cutoff = (1.0 / step).min(1.0);
    let mut out = vec![0f32; frames * 2];
    for i in 0..frames {
        if i % 1024 == 0 {
            cancelled(cancel)?;
        }
        let pos = i as f64 * step;
        let mut sum = [0f64; 2];
        let mut weight = 0f64;
        for tap in -16..=16 {
            let src = pos.floor() as i64 + tap;
            if src < 0 || src >= n as i64 {
                continue;
            }
            let distance = pos - src as f64;
            let x = std::f64::consts::PI * distance * cutoff;
            let sinc = if x.abs() < 1e-12 { 1.0 } else { x.sin() / x };
            let w = sinc * (0.5 + 0.5 * (std::f64::consts::PI * distance / 17.0).cos()) * cutoff;
            weight += w;
            for ch in 0..2 {
                sum[ch] += f64::from(input[src as usize * 2 + ch]) * w;
            }
        }
        for ch in 0..2 {
            out[i * 2 + ch] = (sum[ch] / weight.max(1e-12)) as f32;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tone(rate: u32, hz: f64) -> SampleData {
        SampleData::from_stereo(
            rate,
            (0..rate as usize / 2)
                .flat_map(|i| {
                    let a = (i as f64 * hz * std::f64::consts::TAU / f64::from(rate)).sin() as f32;
                    [a, -a]
                })
                .collect(),
        )
    }
    #[test]
    fn duration_pitch_stereo_and_all_source_rates() {
        for rate in [
            8000, 11025, 16000, 22050, 24000, 32000, 44100, 48000, 88200, 96000, 176400, 192000, 384000,
        ] {
            let source = tone(rate, 440.);
            for ratio in [0.5, 1., 2.] {
                for pitch in [-12., 0., 12.] {
                    let out = prepare(&source, ratio, pitch, true, &AtomicBool::new(false)).unwrap();
                    assert_eq!(out.frames(), (source.frames() as f64 * ratio).round() as usize);
                    assert!(out.data.as_chunks::<2>().0.iter().all(|f| (f[0] + f[1]).abs() < 1e-6));
                    let a = out.frames() / 4;
                    let b = out.frames() * 3 / 4;
                    let crossings = (a..b - 1)
                        .filter(|&i| out.data[i * 2] <= 0. && out.data[(i + 1) * 2] > 0.)
                        .count();
                    let hz = crossings as f64 * f64::from(rate) / (b - a) as f64;
                    let expected = 440. * 2f64.powf(pitch / 12.);
                    assert!((hz - expected).abs() < expected * 0.05, "{rate} {ratio} {pitch}: {hz}");
                }
            }
        }
    }
    #[test]
    fn bounds_identity_silence_tiny_and_cancel() {
        let source = tone(8000, 440.);
        assert_eq!(
            prepare(&source, 1., 0., true, &AtomicBool::new(false)).unwrap().data,
            source.data
        );
        for ratio in [0., 0.49, 2.01, f64::NAN, f64::INFINITY] {
            assert!(prepare(&source, ratio, 0., true, &AtomicBool::new(false)).is_err());
        }
        assert!(prepare(&source, 1., 0., true, &AtomicBool::new(true)).is_err());
        for n in [2, 16, 1024] {
            let s = SampleData::from_stereo(384000, vec![0.; n * 2]);
            assert!(
                prepare(&s, 2., 0., true, &AtomicBool::new(false))
                    .unwrap()
                    .data
                    .iter()
                    .all(|&v| v == 0.)
            );
        }
    }
}
