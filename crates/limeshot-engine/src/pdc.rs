//! Plugin delay compensation (FL Studio's automatic PDC).
//!
//! A plugin that looks ahead (a linear-phase EQ, a limiter) reports a latency: its output lags
//! its input by that many samples. Every path to the master must lag by the same amount, or
//! parallel signals drift apart. For each mixer track, its *arrival* latency `A` is the largest
//! latency of anything that feeds it (plugin channels; tracks routed to it, through their own
//! inserts), and everything that arrives earlier is delayed to match:
//!
//! * the track's voices (samplers, LimeSynth; latency 0) by `A`,
//! * a plugin channel with latency `L` by `A − L`,
//! * a route from track `j` by `A − out(j)`, where `out(j) = A(j) + inserts(j)`, read from one
//!   ring per source track at a tap per route.
//!
//! The master's `out` is the total latency, which exports trim from the start (FL's "Trim PDC
//! silence"). Delay lines come from a pool allocated with the renderer; planning runs on the audio
//! thread without allocating and keeps the rings it already uses, so a new snapshot does not
//! interrupt delayed audio. Paths the pool cannot cover stay uncompensated.

use limeshot_core::Mixer;
use limeshot_sequencer::Snapshot;

use crate::MAX_BLOCK;

/// The longest delay compensated, in frames (about 340 ms at 48 kHz).
pub const MAX_PDC: usize = 1 << 14;
/// Delay lines available. Each holds `MAX_PDC + MAX_BLOCK` stereo frames.
const RINGS: usize = 64;
const TRACKS: usize = Mixer::MAX_TRACKS;
const NONE: u16 = u16::MAX;

pub(crate) struct Ring {
    buf: Vec<f32>,
    /// Next frame to write.
    pos: usize,
}

impl Ring {
    const FRAMES: usize = MAX_PDC + MAX_BLOCK;

    pub(crate) fn new() -> Self {
        Self {
            buf: vec![0.0; 2 * Self::FRAMES],
            pos: 0,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    pub(crate) fn write(&mut self, src: &[f32]) {
        for f in src.as_chunks::<2>().0 {
            self.buf[2 * self.pos] = f[0];
            self.buf[2 * self.pos + 1] = f[1];
            self.pos = (self.pos + 1) % Self::FRAMES;
        }
    }

    /// The `dst.len() / 2` frames written just now, as they were `delay` frames earlier.
    pub(crate) fn read(&self, delay: usize, dst: &mut [f32]) {
        let n = dst.len() / 2;
        let mut at = (self.pos + 2 * Self::FRAMES - n - delay.min(MAX_PDC)) % Self::FRAMES;
        for f in dst.as_chunks_mut::<2>().0 {
            f[0] = self.buf[2 * at];
            f[1] = self.buf[2 * at + 1];
            at = (at + 1) % Self::FRAMES;
        }
    }
}

/// Which delay each path needs (in frames), and the ring holding its history.
#[derive(Clone, Copy, PartialEq)]
struct Line {
    delay: u32,
    ring: u16,
}

impl Line {
    const OFF: Line = Line { delay: 0, ring: NONE };
}

pub(crate) struct Pdc {
    rings: Vec<Ring>,
    used: Vec<bool>,
    /// Per track: its arrival latency `A` and its output latency `out`.
    arrival: Vec<u32>,
    out: Vec<u32>,
    /// Per track: the delay of its voices (`A`).
    voices: Vec<Line>,
    /// Per track: the ring of its post-fader output (routes read it at their taps).
    output: Vec<u16>,
    /// Per snapshot channel index: the delay of its plugin's output.
    channels: Vec<Line>,
    /// Scratch for a delayed route.
    scratch: Vec<f32>,
    total: u32,
    /// The latencies the plan was made for: (inserts per track, plugin per channel).
    seen: Vec<u32>,
    /// Plan with these, to compare with the next plan.
    next_seen: Vec<u32>,
}

impl Pdc {
    pub(crate) fn new() -> Self {
        Self {
            rings: (0..RINGS).map(|_| Ring::new()).collect(),
            used: vec![false; RINGS],
            arrival: vec![0; TRACKS],
            out: vec![0; TRACKS],
            voices: vec![Line::OFF; TRACKS],
            output: vec![NONE; TRACKS],
            channels: vec![Line::OFF; crate::plugins::MAX_PLUGIN_CHANNELS],
            scratch: vec![0.0; 2 * MAX_BLOCK],
            total: 0,
            seen: Vec::with_capacity(TRACKS + crate::plugins::MAX_PLUGIN_CHANNELS),
            next_seen: Vec::with_capacity(TRACKS + crate::plugins::MAX_PLUGIN_CHANNELS),
        }
    }

    /// The total latency in frames: how far the master's output lags the transport.
    pub(crate) fn total(&self) -> u32 {
        self.total
    }

    /// Make the plan for `snapshot` with the given latencies (`inserts(track)`, and
    /// `channel(index)` for plugin channels). Keeps everything as it is when nothing changed.
    pub(crate) fn plan(
        &mut self,
        snapshot: &Snapshot,
        inserts: impl Fn(usize) -> u32,
        channel: impl Fn(usize) -> Option<u32>,
        force: bool,
    ) {
        let tracks = snapshot.mixer.len().min(TRACKS);
        self.next_seen.clear();
        for t in 0..tracks {
            self.next_seen.push(inserts(t));
        }
        for i in 0..snapshot.channels.len().min(self.channels.len()) {
            self.next_seen.push(channel(i).unwrap_or(u32::MAX));
        }
        if !force && self.next_seen == self.seen {
            return;
        }
        std::mem::swap(&mut self.seen, &mut self.next_seen);
        let clamp = |v: u32| v.min(MAX_PDC as u32);

        // Arrival latencies, sources before destinations.
        let mut fed = [false; TRACKS];
        self.arrival[..tracks].fill(0);
        for (i, ch) in snapshot.channels.iter().enumerate() {
            let t = ch.mixer_track.min(tracks.saturating_sub(1));
            fed[t] = true;
            if let Some(l) = self.channel_latency(tracks, i) {
                self.arrival[t] = self.arrival[t].max(l);
            }
        }
        let order = snapshot.mix_order.iter().copied().filter(|&i| i != 0 && i < tracks);
        for i in order.chain(std::iter::once(0)) {
            self.out[i] = clamp(self.arrival[i] + self.seen[i]);
            if i == 0 {
                continue;
            }
            for &(to, _, _) in &snapshot.mixer[i].routes {
                if to < tracks && to != i {
                    // A track with nothing feeding it carries only silence: no compensation.
                    if fed[i] {
                        fed[to] = true;
                        self.arrival[to] = self.arrival[to].max(self.out[i]);
                    }
                }
            }
        }
        self.total = self.out[0];

        // Rings: keep the ones still needed, release the rest, then give out new ones.
        let mut keep = [false; RINGS];
        let mut voices = [Line::OFF; TRACKS];
        let mut output = [NONE; TRACKS];
        let mut needs_output = [false; TRACKS];
        for t in 0..tracks {
            if self.arrival[t] > 0 {
                voices[t] = Line {
                    delay: self.arrival[t],
                    ring: self.voices[t].ring,
                };
            }
            // A route needs a delayed copy of this track when its destination waits longer.
            needs_output[t] = fed[t]
                && t != 0
                && snapshot.mixer[t]
                    .routes
                    .iter()
                    .any(|&(to, _, _)| to < tracks && to != t && self.arrival[to] > self.out[t]);
            if needs_output[t] {
                output[t] = self.output[t];
            }
        }
        let mut channels = [Line::OFF; crate::plugins::MAX_PLUGIN_CHANNELS];
        for (i, ch) in snapshot.channels.iter().enumerate().take(channels.len()) {
            let t = ch.mixer_track.min(tracks.saturating_sub(1));
            if let Some(l) = self.channel_latency(tracks, i)
                && self.arrival[t] > l
            {
                channels[i] = Line {
                    delay: self.arrival[t] - l,
                    ring: self.channels[i].ring,
                };
            }
        }
        for r in voices
            .iter()
            .map(|l| l.ring)
            .chain(output.iter().copied())
            .chain(channels.iter().map(|l| l.ring))
        {
            if let Some(k) = keep.get_mut(usize::from(r)) {
                *k = true;
            }
        }
        for (u, k) in self.used.iter_mut().zip(keep) {
            *u = k;
        }
        let take = |used: &mut Vec<bool>, rings: &mut Vec<Ring>| -> u16 {
            match used.iter().position(|u| !u) {
                Some(i) => {
                    used[i] = true;
                    rings[i].clear();
                    i as u16
                }
                None => NONE,
            }
        };
        for l in voices.iter_mut().filter(|l| l.delay > 0 && l.ring == NONE) {
            l.ring = take(&mut self.used, &mut self.rings);
        }
        for (r, needs) in output.iter_mut().zip(needs_output) {
            if needs && *r == NONE {
                *r = take(&mut self.used, &mut self.rings);
            }
        }
        for l in channels.iter_mut().filter(|l| l.delay > 0 && l.ring == NONE) {
            l.ring = take(&mut self.used, &mut self.rings);
        }
        self.voices[..tracks].copy_from_slice(&voices[..tracks]);
        self.voices[tracks..].fill(Line::OFF);
        self.output[..tracks].copy_from_slice(&output[..tracks]);
        self.output[tracks..].fill(NONE);
        self.channels.copy_from_slice(&channels);
    }

    fn channel_latency(&self, tracks: usize, i: usize) -> Option<u32> {
        match self.seen.get(tracks + i) {
            Some(&u32::MAX) | None => None,
            Some(&l) => Some(l.min(MAX_PDC as u32)),
        }
    }

    /// Delay track `t`'s voices (its bus before plugin channels are added).
    pub(crate) fn voices(&mut self, t: usize, bus: &mut [f32]) {
        let Some(&Line { delay, ring }) = self.voices.get(t) else {
            return;
        };
        if let Some(r) = self.rings.get_mut(usize::from(ring)) {
            r.write(bus);
            r.read(delay as usize, bus);
        }
    }

    /// Delay the output of the plugin on snapshot channel `i`.
    pub(crate) fn channel(&mut self, i: usize, buf: &mut [f32]) {
        let Some(&Line { delay, ring }) = self.channels.get(i) else {
            return;
        };
        if let Some(r) = self.rings.get_mut(usize::from(ring)) {
            r.write(buf);
            r.read(delay as usize, buf);
        }
    }

    /// Add track `t`'s pre-fader output `src` into `dst` at `(left, right)` gains (the fader
    /// and pan times the send level, or the level alone for a pre-fader send), delayed as the
    /// route to track `to` needs.
    pub(crate) fn route(&mut self, t: usize, to: usize, src: &[f32], dst: &mut [f32], level: (f32, f32)) {
        let delay = self
            .arrival
            .get(to)
            .copied()
            .unwrap_or(0)
            .saturating_sub(self.out.get(t).copied().unwrap_or(0));
        let ring = self.output.get(t).copied().unwrap_or(NONE);
        match self.rings.get(usize::from(ring)) {
            Some(r) if delay > 0 => {
                let n = src.len().min(self.scratch.len());
                r.read(delay as usize, &mut self.scratch[..n]);
                for (d, s) in dst
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip(self.scratch[..n].as_chunks::<2>().0)
                {
                    d[0] += s[0] * level.0;
                    d[1] += s[1] * level.1;
                }
            }
            _ => {
                for (d, s) in dst.as_chunks_mut::<2>().0.iter_mut().zip(src.as_chunks::<2>().0) {
                    d[0] += s[0] * level.0;
                    d[1] += s[1] * level.1;
                }
            }
        }
    }

    /// Record track `t`'s pre-fader output for its delayed routes. Call once per block, before
    /// [`Pdc::route`].
    pub(crate) fn record(&mut self, t: usize, src: &[f32]) {
        let ring = self.output.get(t).copied().unwrap_or(NONE);
        if let Some(r) = self.rings.get_mut(usize::from(ring)) {
            r.write(src);
        }
    }

    /// Silence every delay line (the effects were reset).
    pub(crate) fn clear(&mut self) {
        for (r, &u) in self.rings.iter_mut().zip(&self.used) {
            if u {
                r.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ring_returns_what_was_written_delay_frames_earlier() {
        let mut r = Ring::new();
        let mut got = Vec::new();
        for block in 0..40 {
            let src: Vec<f32> = (0..2 * 100).map(|i| (block * 100 + i / 2) as f32).collect();
            r.write(&src);
            let mut dst = vec![0.0; src.len()];
            r.read(250, &mut dst);
            got.extend(dst.chunks(2).map(|f| f[0]));
        }
        for (i, v) in got.iter().enumerate().skip(250) {
            assert_eq!(*v, (i - 250) as f32);
        }
        assert!(got[..250].iter().all(|&v| v == 0.0));
    }
}
