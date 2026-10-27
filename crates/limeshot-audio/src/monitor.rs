//! Bounded live-input handoff. Device callbacks exchange complete stereo frames without locks.
//! Linear conversion handles different device rates; excess backlog is discarded rather than
//! letting monitoring latency grow without bound. This is not recording latency compensation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Clone)]
pub struct Control(Arc<AtomicU64>);

impl Control {
    /// None mutes; a track routes before its inserts and fader. A change invalidates queued audio.
    pub fn set_track(&self, track: Option<u16>) {
        let route = track
            .filter(|t| usize::from(*t) < limeshot_core::Mixer::MAX_TRACKS)
            .map_or(0, |t| u64::from(t) + 1);
        let old = self.0.load(Ordering::Relaxed);
        if old & 0xffff != route {
            self.0
                .store(((old >> 16).wrapping_add(1) << 16) | route, Ordering::Release);
        }
    }
}

pub struct Sender {
    queue: rtrb::Producer<(u64, [f32; 2])>,
    control: Control,
    overflow: Arc<AtomicBool>,
}

impl Sender {
    pub fn control(&self) -> Control {
        self.control.clone()
    }

    pub(crate) fn push(&mut self, frame: [f32; 2]) {
        let state = self.control.0.load(Ordering::Acquire);
        if state & 0xffff != 0 && self.queue.push((state, frame)).is_err() {
            self.overflow.store(true, Ordering::Release);
        }
    }
}

pub struct Receiver {
    queue: rtrb::Consumer<(u64, [f32; 2])>,
    control: Control,
    overflow: Arc<AtomicBool>,
    state: u64,
    ratio: f64,
    phase: f64,
    current: Option<[f32; 2]>,
    next: Option<[f32; 2]>,
    max_backlog: usize,
}

pub(crate) fn channel(input_rate: u32, output_rate: u32) -> (Sender, Box<Receiver>) {
    let rate = input_rate.clamp(1, 768_000);
    let (tx, rx) = rtrb::RingBuffer::new((rate as usize / 10).max(4));
    let control = Control(Arc::new(AtomicU64::new(0)));
    let overflow = Arc::new(AtomicBool::new(false));
    (
        Sender {
            queue: tx,
            control: control.clone(),
            overflow: overflow.clone(),
        },
        Box::new(Receiver {
            queue: rx,
            control,
            overflow,
            state: 0,
            ratio: f64::from(rate) / f64::from(output_rate.max(1)),
            phase: 0.0,
            current: None,
            next: None,
            max_backlog: (rate as usize / 50).max(2),
        }),
    )
}

impl Receiver {
    fn reset(&mut self) {
        self.current = None;
        self.next = None;
        self.phase = 0.0;
    }

    fn pop(&mut self) -> Option<[f32; 2]> {
        for _ in 0..self.queue.slots() {
            let Ok((state, frame)) = self.queue.pop() else {
                break;
            };
            if state == self.state {
                return Some(frame);
            }
        }
        None
    }

    /// Fill one output block; missing input is silence, never a repeated last sample.
    pub(crate) fn read(&mut self, out: &mut [f32]) -> Option<u16> {
        out.fill(0.0);
        let state = self.control.0.load(Ordering::Acquire);
        if self.state != state {
            self.state = state;
            self.reset();
        }
        let queued = self.queue.slots();
        // One bounded discard per block also recovers from input/output clock drift or a pause.
        let overflow = self.overflow.swap(false, Ordering::Acquire);
        if overflow || state & 0xffff == 0 {
            let needed = ((out.len() / 2) as f64 * self.ratio).ceil() as usize + 1;
            let discard = if state & 0xffff == 0 {
                queued
            } else {
                queued.saturating_sub(self.max_backlog.max(needed))
            };
            for _ in 0..discard {
                let _ = self.queue.pop();
            }
            self.reset();
        }
        let route = (state & 0xffff) as u16;
        if route == 0 {
            return None;
        }
        for frame in out.as_chunks_mut::<2>().0 {
            if self.current.is_none() {
                self.current = self.pop();
            }
            while self.phase >= 1.0 && self.current.is_some() {
                self.current = self.next.take().or_else(|| self.pop());
                self.phase -= 1.0;
            }
            let Some(a) = self.current else {
                self.reset();
                break;
            };
            if self.phase > 0.0 && self.next.is_none() {
                self.next = self.pop();
                if self.next.is_none() {
                    self.reset();
                    break;
                }
            }
            let b = self.next.unwrap_or(a);
            for c in 0..2 {
                frame[c] = a[c] + (b[c] - a[c]) * self.phase as f32;
            }
            self.phase += self.ratio;
        }
        Some(route - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mute_reroute_overflow_and_underflow_do_not_replay_stale_frames() {
        let (mut tx, mut rx) = channel(100, 100);
        let control = tx.control();
        control.set_track(Some(1));
        tx.push([0.25, -0.5]);
        let mut out = [9.0; 4];
        assert_eq!(rx.read(&mut out), Some(1));
        assert_eq!(out, [0.25, -0.5, 0.0, 0.0]);
        tx.push([0.75; 2]);
        control.set_track(None);
        assert_eq!(rx.read(&mut out), None);
        assert_eq!(out, [0.0; 4]);
        control.set_track(Some(2));
        tx.push([0.5; 2]);
        control.set_track(Some(3));
        tx.push([-0.25; 2]);
        assert_eq!(rx.read(&mut out), Some(3));
        assert_eq!(out, [-0.25, -0.25, 0.0, 0.0]);
        for i in 0..100 {
            tx.push([i as f32; 2]);
        }
        rx.read(&mut out);
        assert_eq!(out, [7.0, 7.0, 8.0, 8.0]);
    }

    #[test]
    fn conversion_is_continuous_across_output_block_sizes() {
        for (input, output) in [(48_000, 48_000), (44_100, 48_000), (96_000, 44_100)] {
            for block in [1, 7, 64, 333] {
                let (mut tx, mut rx) = channel(input, output);
                tx.control().set_track(Some(0));
                for i in 0..700 {
                    tx.push([i as f32 / 1000.0, -(i as f32) / 1000.0]);
                }
                let mut samples = vec![0.0; 600];
                for chunk in samples.chunks_mut(block * 2) {
                    rx.read(chunk);
                }
                for (i, frame) in samples.as_chunks::<2>().0.iter().enumerate() {
                    let expected = i as f32 * input as f32 / output as f32 / 1000.0;
                    assert!(
                        (frame[0] - expected).abs() < 1e-5,
                        "{input}/{output}, block {block}, frame {i}"
                    );
                    assert!((frame[1] + expected).abs() < 1e-5);
                }
            }
        }
    }
}
