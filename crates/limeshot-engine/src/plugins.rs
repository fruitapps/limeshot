//! Plugin instruments on the audio thread.
//!
//! A plugin channel has no voices: it gets the block's notes as events at frame offsets and
//! renders the whole block itself. Instruments are built on the control thread and handed over
//! with [`InstrumentRack::adopt`]; [`InstrumentRack::arrange`] places them by channel id when a
//! snapshot arrives, like the effect rack. Everything is preallocated.

use limeshot_core::Tick;
use limeshot_dsp::{Instrument, NoteEvent, NoteEventKind, Transport};
use limeshot_sequencer::{InstrumentState, Snapshot};

/// Plugin instruments that can play at once (channels beyond this index stay silent).
pub const MAX_PLUGIN_CHANNELS: usize = 256;
/// Held notes across all plugin channels.
const MAX_HELD: usize = 1024;
/// Note events per channel per block.
const MAX_EVENTS: usize = 512;

struct Slot {
    channel: u32,
    inst: Box<dyn Instrument>,
}

/// A note that is sounding, waiting for its note-off.
#[derive(Clone, Copy)]
struct Held {
    channel: u32,
    key: u8,
    id: i32,
    /// Engine sample of the note-on: its note-off goes out at least one frame later, so the
    /// plugin never gets the off before the on (an audition released in the block it started).
    on_at: u64,
    /// Engine sample of the note-off; `u64::MAX` until released.
    off_at: u64,
    end_tick: Tick,
    /// Started by the sequencer (released on stop), not by an audition.
    sequenced: bool,
}

pub(crate) struct InstrumentRack {
    /// By snapshot channel index.
    slots: Vec<Option<Slot>>,
    pool: Vec<Slot>,
    retired: Vec<Box<dyn Instrument>>,
    /// Pending events of each slot this block.
    events: Vec<Vec<NoteEvent>>,
    held: Vec<Held>,
    next_id: i32,
    buf: Vec<f32>,
}

impl InstrumentRack {
    pub(crate) fn new(max_block: usize) -> Self {
        Self {
            slots: (0..MAX_PLUGIN_CHANNELS).map(|_| None).collect(),
            pool: Vec::with_capacity(MAX_PLUGIN_CHANNELS),
            retired: Vec::with_capacity(2 * MAX_PLUGIN_CHANNELS),
            events: (0..MAX_PLUGIN_CHANNELS)
                .map(|_| Vec::with_capacity(MAX_EVENTS))
                .collect(),
            held: Vec::with_capacity(MAX_HELD),
            next_id: 0,
            buf: vec![0.0; 2 * max_block],
        }
    }

    pub(crate) fn adopt(&mut self, channel: u32, inst: Box<dyn Instrument>) {
        if let Some(old) = self.pool.iter().position(|s| s.channel == channel) {
            let s = self.pool.swap_remove(old);
            self.retire(s.inst);
        }
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(Slot { channel, inst });
        } else {
            self.retire(inst);
        }
    }

    fn retire(&mut self, inst: Box<dyn Instrument>) {
        if self.retired.len() < self.retired.capacity() {
            self.retired.push(inst);
        }
        // Otherwise it drops here; only reachable if the control thread stops collecting.
    }

    /// Place instruments by channel id as `snapshot` lays the channels out.
    pub(crate) fn arrange(&mut self, snapshot: &Snapshot) {
        for s in &mut self.slots {
            if let Some(slot) = s.take() {
                if self.pool.len() < self.pool.capacity() {
                    self.pool.push(slot);
                } else {
                    self.retired.push(slot.inst);
                }
            }
        }
        for (i, ch) in snapshot.channels.iter().enumerate().take(MAX_PLUGIN_CHANNELS) {
            if let InstrumentState::Plugin { channel } = ch.instrument
                && let Some(p) = self.pool.iter().position(|s| s.channel == channel)
            {
                self.slots[i] = Some(self.pool.swap_remove(p));
            }
        }
        // Instruments whose channel left (or changed instrument) are retired; a replacement
        // adopted before this snapshot is already placed.
        while let Some(s) = self.pool.pop() {
            self.retire(s.inst);
        }
        let live = |c: u32| {
            snapshot
                .channels
                .iter()
                .any(|ch| matches!(ch.instrument, InstrumentState::Plugin { channel } if channel == c))
        };
        self.held.retain(|h| live(h.channel));
    }

    pub(crate) fn clear(&mut self) {
        for i in 0..self.slots.len() {
            if let Some(s) = self.slots[i].take() {
                self.retire(s.inst);
            }
        }
        while let Some(s) = self.pool.pop() {
            self.retire(s.inst);
        }
        self.held.clear();
    }

    pub(crate) fn take_retired(&mut self) -> Option<Box<dyn Instrument>> {
        self.retired.pop()
    }

    pub(crate) fn keep_retired(&mut self, inst: Box<dyn Instrument>) {
        self.retire(inst);
    }

    /// Set parameter `id` of the plugin on snapshot channel `channel` at frame `offset` of the next
    /// block (automation).
    pub(crate) fn set_plugin_param(&mut self, channel: usize, id: u32, offset: u32, value: f32) {
        if let Some(Some(s)) = self.slots.get_mut(channel) {
            s.inst.set_plugin_param(id, offset, f64::from(value));
        }
    }

    /// The latency of the plugin on snapshot channel `channel`, if it has one.
    pub(crate) fn latency(&self, channel: usize) -> Option<u32> {
        self.slots.get(channel)?.as_ref().map(|s| s.inst.latency())
    }

    pub(crate) fn has_instruments(&self) -> bool {
        self.slots.iter().any(Option::is_some)
    }

    /// Start a note on snapshot channel `index` (plugin channel `channel`) at frame `offset` of
    /// the block, engine sample `at`. `off_at` is the engine sample of its note-off (`u64::MAX`:
    /// until released).
    pub(crate) fn note_on(
        &mut self,
        index: usize,
        channel: u32,
        key: u8,
        velocity: f32,
        (offset, at): (usize, u64),
        off_at: u64,
        end_tick: Tick,
        sequenced: bool,
    ) {
        if self.slots.get(index).and_then(Option::as_ref).is_none() {
            return;
        }
        // The same key started at the same sample already sounds.
        if self
            .held
            .iter()
            .any(|h| h.channel == channel && h.key == key && h.on_at == at)
        {
            return;
        }
        // A key that is still held ends where it is played again, as on a keyboard.
        for h in self.held.iter_mut().filter(|h| h.channel == channel && h.key == key) {
            h.off_at = h.off_at.min(at);
        }
        if self.held.len() >= self.held.capacity() {
            return;
        }
        self.next_id = self.next_id.wrapping_add(1) & i32::MAX;
        let id = self.next_id;
        let ev = NoteEvent {
            offset: offset as u32,
            kind: NoteEventKind::On {
                key,
                velocity: velocity.clamp(0.0, 1.0),
                id,
            },
        };
        if let Some(events) = self.events.get_mut(index)
            && events.len() < events.capacity()
        {
            events.push(ev);
            self.held.push(Held {
                channel,
                key,
                id,
                on_at: at,
                off_at,
                end_tick,
                sequenced,
            });
        }
    }

    /// Release an auditioned key.
    pub(crate) fn preview_off(&mut self, channel: u32, key: u8) {
        for h in self
            .held
            .iter_mut()
            .filter(|h| !h.sequenced && h.channel == channel && h.key == key)
        {
            h.off_at = 0;
        }
    }

    /// Release every sequenced note at the start of the next block (stop, locate, loop).
    pub(crate) fn release_sequenced(&mut self) {
        for h in self.held.iter_mut().filter(|h| h.sequenced) {
            h.off_at = 0;
        }
    }

    /// End every note and reset every plugin (its tails, delay lines and voices): each gets its
    /// note-offs in a one-frame block first, then `reset`. Notes queued for this block are dropped.
    pub(crate) fn silence(&mut self, transport: &Transport) {
        for i in 0..self.slots.len() {
            let Some(slot) = self.slots[i].as_mut() else { continue };
            let events = &mut self.events[i];
            events.clear();
            for h in self.held.iter().filter(|h| h.channel == slot.channel) {
                if events.len() < MAX_EVENTS {
                    events.push(NoteEvent {
                        offset: 0,
                        kind: NoteEventKind::Off { key: h.key, id: h.id },
                    });
                }
            }
            if !events.is_empty() {
                slot.inst.process(&mut self.buf[..2], events, transport);
                events.clear();
            }
            slot.inst.reset();
        }
        self.held.clear();
    }

    pub(crate) fn release_all(&mut self) {
        for h in &mut self.held {
            h.off_at = 0;
        }
    }

    /// Move note-offs after a tempo change (`sample_of` maps a tick to an engine sample).
    pub(crate) fn retime(&mut self, sample_of: impl Fn(Tick) -> u64) {
        for h in self
            .held
            .iter_mut()
            .filter(|h| h.sequenced && h.off_at != u64::MAX && h.off_at != 0)
        {
            h.off_at = sample_of(h.end_tick);
        }
    }

    /// Render every plugin channel for the `frames` frames starting at engine sample `start` and
    /// add each into its mixer bus. `mix(i)` gives channel `i`'s (bus, left gain, right gain),
    /// or `None` when it is silent.
    pub(crate) fn render(
        &mut self,
        start: u64,
        frames: usize,
        transport: &Transport,
        buses: &mut [Vec<f32>],
        pdc: &mut crate::pdc::Pdc,
        mix: impl Fn(usize) -> Option<(usize, f32, f32)>,
    ) {
        let n = (2 * frames).min(self.buf.len());
        for i in 0..self.slots.len() {
            let Some(channel) = self.slots[i].as_ref().map(|s| s.channel) else {
                continue;
            };
            let end = start + frames as u64;
            // Note-offs due in this block, at their offsets.
            let mut k = 0;
            while k < self.held.len() {
                let h = self.held[k];
                let due = h.off_at.max(h.on_at + 1);
                if h.channel == channel && due < end && self.events[i].len() < MAX_EVENTS {
                    let offset = due.saturating_sub(start).min(frames.saturating_sub(1) as u64) as u32;
                    self.events[i].push(NoteEvent {
                        offset,
                        kind: NoteEventKind::Off { key: h.key, id: h.id },
                    });
                    self.held.swap_remove(k);
                } else {
                    k += 1;
                }
            }
            sort_events(&mut self.events[i]);
            let Some(slot) = self.slots[i].as_mut() else { continue };
            slot.inst.process(&mut self.buf[..n], &self.events[i], transport);
            self.events[i].clear();
            pdc.channel(i, &mut self.buf[..n]);
            if let Some((bus, gl, gr)) = mix(i)
                && let Some(bus) = buses.get_mut(bus)
            {
                for (f, s) in self.buf[..n].as_chunks::<2>().0.iter().enumerate() {
                    bus[2 * f] += s[0] * gl;
                    bus[2 * f + 1] += s[1] * gr;
                }
            }
        }
    }
}

/// Sort by offset, note-offs before note-ons at the same offset (a retriggered key). Insertion
/// sort: the lists are short and nearly sorted, and it does not allocate.
fn sort_events(events: &mut [NoteEvent]) {
    let rank = |e: &NoteEvent| (e.offset, matches!(e.kind, NoteEventKind::On { .. }));
    for i in 1..events.len() {
        let mut j = i;
        while j > 0 && rank(&events[j - 1]) > rank(&events[j]) {
            events.swap(j - 1, j);
            j -= 1;
        }
    }
}
