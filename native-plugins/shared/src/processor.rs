use crate::effects::build;
use limeshot_dsp::voice::{SynthMods, Voice, VoiceInstrument as InstrumentState};
use limeshot_dsp::{Adsr, SampleData, db_to_gain, effects::Effect, filter::Mode, osc::Shape};
use nice_plug::editor::dpi::LogicalSize;
use nice_plug::midi::{Channel, Key, VoiceID};
use nice_plug::prelude::*;
use nice_plug_egui::{EguiEditor, EguiEditorState, EguiNiceSettings, RepaintNotifier, create_egui_editor};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::{
    editor::NativeEditor,
    parameters::{self, MAX_CONTROLS, NativeParams},
};

pub struct Meters {
    pub input: AtomicU32,
    pub output: AtomicU32,
    pub reduction: AtomicU32,
    pub scope: [AtomicU32; 128],
    pub preview_note: AtomicU32,
    pub sample_rate: AtomicU32,
}
impl Default for Meters {
    fn default() -> Self {
        Self {
            input: AtomicU32::new(0),
            output: AtomicU32::new(0),
            reduction: AtomicU32::new(0),
            scope: std::array::from_fn(|_| AtomicU32::new(0)),
            preview_note: AtomicU32::new(0),
            sample_rate: AtomicU32::new(48000.0_f32.to_bits()),
        }
    }
}
impl Meters {
    /// Read an atomically shared float stored as IEEE 754 bits.
    pub fn read(value: &AtomicU32) -> f32 {
        f32::from_bits(value.load(Ordering::Relaxed))
    }
    fn store(value: &AtomicU32, v: f32) {
        value.store(v.to_bits(), Ordering::Relaxed);
    }
}

#[derive(Default)]
struct Slot {
    voice: Voice,
    id: Option<i32>,
    channel: u8,
    velocity: f32,
    held: bool,
    sustained: bool,
    serial: u64,
    clip: bool,
    pan: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipSysEx {
    Clip(limeshot_dsp::ClipMessage),
    Telemetry(limeshot_dsp::PluginTelemetry),
    ChannelPan(f32),
}
impl SysExMessage for ClipSysEx {
    type Buffer = [u8; limeshot_dsp::CLIP_MESSAGE_LEN];
    /// Validate supported Lime protocol messages and ignore unrelated SysEx.
    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        limeshot_dsp::ClipMessage::decode(buffer)
            .map(Self::Clip)
            .or_else(|| limeshot_dsp::instrument::decode_channel_pan(buffer).map(Self::ChannelPan))
    }
    /// Encode bounded native protocol messages into stack-owned host event storage.
    fn as_buffer(&self) -> (Self::Buffer, usize) {
        match self {
            Self::ChannelPan(pan) => {
                let mut bytes = [0; limeshot_dsp::CLIP_MESSAGE_LEN];
                bytes[..14].copy_from_slice(&limeshot_dsp::instrument::channel_pan_message(*pan));
                (bytes, 14)
            }
            Self::Clip(message) => (message.encode(), limeshot_dsp::CLIP_MESSAGE_LEN),
            Self::Telemetry(value) => {
                let mut bytes = [0; limeshot_dsp::CLIP_MESSAGE_LEN];
                bytes[..22].copy_from_slice(&value.encode());
                (bytes, 22)
            }
        }
    }
}

pub struct LimePlugin<const ID: u8> {
    pub(crate) params: Arc<NativeParams>,
    pub(crate) meters: Arc<Meters>,
    editor_state: Arc<EguiEditorState>,
    notifier: RepaintNotifier,
    effect: Option<Box<dyn Effect>>,
    sample: Option<Arc<SampleData>>,
    active_end: usize,
    offline: bool,
    /// Defensive overflow retention: never release the last sample reference in process().
    retired_sample: Option<Arc<SampleData>>,
    voices: [Slot; 128],
    bend: [f32; 16],
    pedal: [bool; 16],
    serial: u64,
    sample_rate: f32,
    last_values: [f32; MAX_CONTROLS],
    scope_offset: usize,
    preview_note: u32,
    meter_decay: f32,
    block_reduction: f32,
    channel_pan: f32,
}

impl<const ID: u8> Default for LimePlugin<ID> {
    /// Create an inactive plugin with default parameters; activation prepares its sample and DSP.
    fn default() -> Self {
        Self {
            params: Arc::new(NativeParams::new(ID)),
            meters: Arc::new(Meters::default()),
            editor_state: {
                let (w, h) = crate::editor::editor_size(ID);
                EguiEditorState::from_size(LogicalSize::new(w, h), 1.0)
            },
            notifier: RepaintNotifier::new(),
            effect: None,
            sample: None,
            active_end: 0,
            offline: false,
            retired_sample: None,
            voices: std::array::from_fn(|_| Slot::default()),
            bend: [0.0; 16],
            pedal: [false; 16],
            serial: 0,
            sample_rate: 48000.0,
            last_values: [f32::NAN; MAX_CONTROLS],
            scope_offset: 0,
            preview_note: 0,
            meter_decay: 0.999,
            block_reduction: 0.0,
            channel_pan: 0.0,
        }
    }
}

impl<const ID: u8> LimePlugin<ID> {
    /// Prepare DSP for an output rate in Hz and refresh the cached embedded sample.
    /// Allocates effect state, acquires the sample-store lock, and clears sounding notes and meters;
    /// call outside audio processing.
    pub(crate) fn prepare(&mut self, rate: f32) {
        self.sample_rate = rate;
        Meters::store(&self.meters.sample_rate, rate);
        self.meter_decay = (-1.0 / (rate * 0.18)).exp();
        while self.params.sample.take_audio().is_some() {}
        self.sample = self.params.sample.audio();
        self.effect = if ID >= 2 {
            Some(build(
                parameters::KINDS[(ID - 2) as usize],
                rate as u32,
                &self.params.values()[..parameters::specs(ID).len()],
            ))
        } else {
            None
        };
        self.clear();
    }
    /// Apply the editor's audition request at a block boundary using current plain controls.
    /// Zero releases the previous preview; 1–128 select MIDI keys 0–127. Effects ignore requests.
    fn sync_preview(&mut self, values: &[f32; MAX_CONTROLS]) {
        let requested = self.meters.preview_note.load(Ordering::Relaxed).min(128);
        if ID >= 2 || requested == self.preview_note {
            return;
        }
        if self.preview_note > 0 {
            self.event(
                NoteEvent::NoteOff {
                    timing: 0,
                    voice_id: VoiceID::ID(i32::MIN),
                    channel: Channel::Number(0),
                    key: Key::Number((self.preview_note - 1) as u8),
                    velocity: 0.0,
                },
                values,
            );
        }
        if requested > 0 {
            self.event(
                NoteEvent::NoteOn {
                    timing: 0,
                    voice_id: VoiceID::ID(i32::MIN),
                    channel: Channel::Number(0),
                    key: Key::Number((requested - 1) as u8),
                    velocity: 0.8,
                },
                values,
            );
        }
        self.preview_note = requested;
    }

    /// Adopt decoded audio without locks or sample destruction. Clear voices while `sample`
    /// still owns the old audio, then transfer that ownership to the control-thread retire queue.
    pub(crate) fn refresh_sample(&mut self) {
        if let Some(old) = self.retired_sample.take()
            && let Err(old) = self.params.sample.retire_audio(old)
        {
            self.retired_sample = Some(old);
            return;
        }
        if self.params.sample.retirement_full() {
            return;
        }
        if let Some(audio) = self.params.sample.take_audio() {
            self.clear();
            if let Some(disk) = audio.as_ref().and_then(|s| s.disk.as_ref()) {
                disk.set_offline(self.offline);
            }
            if let Some(old) = std::mem::replace(&mut self.sample, audio)
                && let Err(old) = self.params.sample.retire_audio(old)
            {
                self.retired_sample = Some(old);
            }
        }
    }

    /// Stop voices and audition, clear pedal/bend state, and reset DSP and meters.
    /// Retains the prepared sample, output rate, and parameters.
    fn clear(&mut self) {
        self.active_end = 0;
        self.preview_note = 0;
        self.meters.preview_note.store(0, Ordering::Relaxed);
        for v in &mut self.voices {
            v.voice.source = limeshot_dsp::voice::Source::Off;
            v.held = false;
            v.sustained = false;
        }
        self.bend.fill(0.0);
        self.pedal.fill(false);
        self.last_values.fill(f32::NAN);
        if let Some(e) = &mut self.effect {
            e.reset();
        }
        Meters::store(&self.meters.input, 0.0);
        Meters::store(&self.meters.output, 0.0);
        Meters::store(&self.meters.reduction, 0.0);
        for point in &self.meters.scope {
            Meters::store(point, 0.0);
        }
    }

    /// Convert plain controls in specification order into shared instrument settings.
    /// Converts dB to gain and milliseconds to seconds; sampler audio uses the prepared cache.
    fn instrument(&self, p: &[f32; MAX_CONTROLS]) -> InstrumentState {
        if ID == 0 {
            InstrumentState::Synth {
                shape: match p[0].round() as u8 {
                    0 => Shape::Sine,
                    1 => Shape::Square,
                    3 => Shape::Triangle,
                    _ => Shape::Saw,
                },
                octave: p[1].round() as i8,
                detune_cents: p[2],
                gain: db_to_gain(p[3]),
                adsr: Adsr {
                    attack: p[4] / 1000.0,
                    decay: p[5] / 1000.0,
                    sustain: p[6],
                    release: p[7] / 1000.0,
                },
                mods: SynthMods {
                    cutoff: p[8],
                    resonance: p[9],
                    env_octaves: p[10],
                    env_decay: p[11] / 1000.0,
                    lfo_rate: p[12],
                    vibrato_cents: p[13],
                    lfo_cutoff: p[14],
                    mode: match p[15].round() as u8 {
                        1 => Mode::BandPass,
                        2 => Mode::HighPass,
                        _ => Mode::LowPass,
                    },
                    key_track: p[16],
                },
            }
        } else {
            InstrumentState::Sampler {
                sample: self.sample.clone(),
                gain: db_to_gain(p[0]),
                pitch: p[1],
                start: p[2],
                end: p[3],
                attack: p[8] / 1000.0,
                decay: p[9] / 1000.0,
                sustain: p[10],
                release: p[4] / 1000.0,
                reverse: p[5] >= 0.5,
                one_shot: p[6] >= 0.5,
                looping: (p[13] >= 0.5).then(|| {
                    let (a, b) = (p[14].clamp(0.0, 1.0), p[15].clamp(0.0, 1.0));
                    (a.min(b), a.max(b))
                }),
                ping_pong: p[16] >= 0.5,
                filter_mode: match p[17].round() as u8 {
                    1 => Mode::BandPass,
                    2 => Mode::HighPass,
                    _ => Mode::LowPass,
                },
                // Other filter outputs remain active even at the maximum cutoff.
                filter: (p[11] < 19_999.0 || p[12] > 0.0 || p[17] >= 0.5)
                    .then(|| (p[11].clamp(20.0, 20_000.0), p[12].clamp(0.0, 0.95))),
            }
        }
    }

    /// Apply current plain controls and each channel's pitch bend to active voices without
    /// retriggering, using their original velocities.
    fn update_voices(&mut self, p: &[f32; MAX_CONTROLS]) {
        let state = self.instrument(p);
        let root = if ID == 1 { p[7].round() as u8 } else { 60 };
        for v in &mut self.voices[..self.active_end] {
            if v.voice.is_active() {
                v.voice.update_instrument(
                    &state,
                    root,
                    v.velocity,
                    self.bend[v.channel as usize],
                    self.sample_rate,
                );
            }
        }
    }

    /// Apply a note, choke, pitch-bend, or supported MIDI controller event immediately.
    /// The caller schedules its timestamp; `p` contains plain controls in specification order.
    /// Nonpositive note-on velocity releases a note. At capacity, steal the oldest unheld
    /// voice, then the oldest held voice. Pitch bend spans ±2 semitones; CC 64, 120, and 123
    /// control sustain, all-sound-off, and all-notes-off. Other events are ignored.
    pub(crate) fn event(&mut self, e: NoteEvent<ClipSysEx>, p: &[f32; MAX_CONTROLS]) {
        match e {
            NoteEvent::NoteOn {
                voice_id,
                channel: Channel::Number(channel),
                key: Key::Number(note),
                velocity,
                ..
            } if channel < 16 && note < 128 => {
                if velocity <= 0.0 {
                    self.release_note(voice_id, Channel::Number(channel), Key::Number(note), p);
                    return;
                }
                let state = self.instrument(p);
                let root = if ID == 1 { p[7].round() as u8 } else { 60 };
                let Some(mut voice) = Voice::for_instrument(&state, root, note, velocity, self.sample_rate) else {
                    return;
                };
                voice.update_instrument(&state, root, velocity, self.bend[channel as usize], self.sample_rate);
                let index = self
                    .voices
                    .iter()
                    .position(|s| !s.voice.is_active())
                    .unwrap_or_else(|| {
                        self.voices
                            .iter()
                            .enumerate()
                            .min_by_key(|(_, s)| (s.held, s.serial))
                            .map_or(0, |(i, _)| i)
                    });
                self.serial = self.serial.wrapping_add(1);
                self.active_end = self.active_end.max(index + 1);
                self.voices[index] = Slot {
                    voice,
                    id: voice_id.id(),
                    channel,
                    velocity,
                    held: true,
                    sustained: false,
                    serial: self.serial,
                    clip: false,
                    pan: 0.0,
                };
            }
            NoteEvent::MidiSysEx {
                message: ClipSysEx::Clip(message),
                ..
            } if ID == 1 => {
                self.event(
                    NoteEvent::NoteOn {
                        timing: 0,
                        voice_id: VoiceID::ID(message.id),
                        channel: Channel::Number(0),
                        key: Key::Number(message.key),
                        velocity: message.velocity,
                    },
                    p,
                );
                if let Some(slot) = self
                    .voices
                    .iter_mut()
                    .find(|s| s.id == Some(message.id) && s.voice.is_active())
                {
                    let playback = message.playback;
                    slot.clip = true;
                    slot.voice
                        .skip(playback.offset_secs + playback.elapsed_secs, self.sample_rate);
                    slot.voice.fade = Some(limeshot_dsp::voice::ClipFade {
                        in_curve: playback.fade_in_curve,
                        out_curve: playback.fade_out_curve,
                        in_len: f64::from(playback.fade_in_secs) * f64::from(self.sample_rate),
                        pos: playback.elapsed_secs * f64::from(self.sample_rate),
                        out_len: f64::from(playback.fade_out_secs) * f64::from(self.sample_rate),
                        left: playback.remaining_secs * f64::from(self.sample_rate),
                    });
                }
            }
            NoteEvent::MidiSysEx {
                message: ClipSysEx::ChannelPan(pan),
                ..
            } => {
                self.channel_pan = pan;
            }
            NoteEvent::PolyPan {
                voice_id,
                channel,
                key,
                pan,
                ..
            } => {
                for slot in &mut self.voices {
                    if matches_note(slot, voice_id, channel, key) {
                        slot.pan = pan;
                    }
                }
            }
            NoteEvent::NoteOff {
                voice_id, channel, key, ..
            } => self.release_note(voice_id, channel, key, p),
            NoteEvent::Choke {
                voice_id, channel, key, ..
            } => {
                for s in &mut self.voices {
                    if matches_note(s, voice_id, channel, key) {
                        s.voice.source = limeshot_dsp::voice::Source::Off;
                        s.held = false;
                    }
                }
            }
            NoteEvent::MidiPitchBend { channel, value, .. } if channel < 16 => {
                self.bend[channel as usize] = (value.clamp(0.0, 1.0) - 0.5) * 4.0;
                self.update_voices(p);
            }
            NoteEvent::MidiCC { channel, cc, value, .. } if channel < 16 => match cc {
                64 => {
                    let down = value >= 0.5;
                    self.pedal[channel as usize] = down;
                    if !down {
                        for s in &mut self.voices {
                            if s.channel == channel && s.sustained {
                                s.voice.release();
                                s.sustained = false;
                            }
                        }
                    }
                }
                120 | 123 => {
                    for s in &mut self.voices {
                        if s.channel == channel {
                            if cc == 120 {
                                s.voice.source = limeshot_dsp::voice::Source::Off;
                            } else {
                                s.voice.release();
                            }
                            s.held = false;
                            s.sustained = false;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Release the oldest held voice matching the note selectors, respecting sustain.
    /// Sampler one-shot mode only marks it unheld; unmatched selectors do nothing.
    fn release_note(&mut self, id: VoiceID, channel: Channel, key: Key, p: &[f32; MAX_CONTROLS]) {
        // Hosts without note IDs pair repeated keys in start order.
        let index = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, s)| s.held && matches_note(s, id, channel, key))
            .min_by_key(|(_, s)| s.serial)
            .map(|(i, _)| i);
        if let Some(i) = index {
            let s = &mut self.voices[i];
            s.held = false;
            if s.clip {
                s.voice.source = limeshot_dsp::voice::Source::Off;
                return;
            }
            // One-shot notes ignore their note-off, unless they loop (as in LimeShot's renderer).
            if ID == 1 && p[6] >= 0.5 && p[13] < 0.5 {
                return;
            }
            s.sustained = self.pedal.get(s.channel as usize).copied().unwrap_or(false);
            if !s.sustained {
                s.voice.release();
            }
        }
    }

    /// Render one stereo frame using plain controls in specification order, updating meters.
    /// `key` supplies an optional external sidechain frame; `tempo` is host BPM. Synced delay
    /// time is clamped to 10–2,000 ms; invalid tempo leaves the manual time in use.
    /// Bypass returns input for effects and silence for instruments while DSP/voices advance.
    pub(crate) fn frame(
        &mut self,
        input: [f32; 2],
        key: Option<[f32; 2]>,
        p: &[f32; MAX_CONTROLS],
        tempo: f64,
    ) -> [f32; 2] {
        let mut out = if ID < 2 {
            if *p != self.last_values {
                self.update_voices(p);
                self.last_values = *p;
            }
            let mut sum = [0.0; 2];
            for s in &mut self.voices[..self.active_end] {
                if s.voice.is_active() {
                    let (l, r) = s.voice.next_frame();
                    let (gl, gr) = limeshot_dsp::pan_gains((s.pan + self.channel_pan).clamp(-1.0, 1.0));
                    sum[0] += l * gl;
                    sum[1] += r * gr;
                }
            }
            while self.active_end > 0 && !self.voices[self.active_end - 1].voice.is_active() {
                self.active_end -= 1;
            }
            sum
        } else {
            input
        };
        if let Some(effect) = &mut self.effect {
            let mut resolved = *p;
            if ID == 5 {
                const BEATS: [f64; 9] = [0.0, 0.25, 1.0 / 3.0, 0.5, 0.75, 1.0, 1.5, 2.0, 4.0];
                let choice = p[5].round() as usize;
                if let Some(beats) = BEATS.get(choice)
                    && choice > 0
                    && tempo.is_finite()
                    && tempo > 0.0
                {
                    resolved[0] = (beats * 60000.0 / tempo).clamp(10.0, 2000.0) as f32;
                }
            }
            if resolved != self.last_values {
                effect.set_params(&resolved[..parameters::specs(ID).len()]);
                self.last_values = resolved;
            }
            if self.params.external_key.value()
                && let Some(key) = key
            {
                effect.process_sidechain(&mut out, &key);
            } else {
                effect.process(&mut out);
            }
            Meters::store(&self.meters.reduction, effect.gain_reduction_db());
        }
        if self.params.bypass.value() {
            out = if ID < 2 { [0.0; 2] } else { input };
        }
        let in_peak = input[0].abs().max(input[1].abs());
        let out_peak = out[0].abs().max(out[1].abs());
        Meters::store(
            &self.meters.input,
            in_peak.max(Meters::read(&self.meters.input) * self.meter_decay),
        );
        Meters::store(
            &self.meters.output,
            out_peak.max(Meters::read(&self.meters.output) * self.meter_decay),
        );
        if self.scope_offset.is_multiple_of(16) {
            Meters::store(
                &self.meters.scope[(self.scope_offset / 16) % 128],
                (out[0] + out[1]) * 0.5,
            );
        }
        self.scope_offset = self.scope_offset.wrapping_add(1);
        out
    }
}

/// Match all supplied note selectors; a wildcard in any selector imposes no constraint.
fn matches_note(s: &Slot, id: VoiceID, channel: Channel, key: Key) -> bool {
    channel.number().is_none_or(|c| s.channel == c)
        && key.number().is_none_or(|k| s.voice.key == k)
        && id.id().is_none_or(|id| s.id == Some(id))
}

impl<const ID: u8> Plugin for LimePlugin<ID> {
    const NAME: &'static str = parameters::NAMES[ID as usize];
    const VENDOR: &'static str = "LimeShot";
    const URL: &'static str = "https://github.com/fruitapps/limeshot";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = if ID < 2 {
        &[AudioIOLayout {
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        }]
    } else if ID == 7 {
        &[
            AudioIOLayout {
                main_input_channels: NonZeroU32::new(2),
                main_output_channels: NonZeroU32::new(2),
                aux_input_ports: &[new_nonzero_u32(2)],
                ..AudioIOLayout::const_default()
            },
            AudioIOLayout {
                main_input_channels: NonZeroU32::new(2),
                main_output_channels: NonZeroU32::new(2),
                ..AudioIOLayout::const_default()
            },
        ]
    } else {
        &[AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        }]
    };
    const MIDI_INPUT: MidiConfig = if ID < 2 { MidiConfig::MidiCCs } else { MidiConfig::None };
    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type Editor = EguiEditor<NativeEditor>;
    type SysExMessage = ClipSysEx;
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    /// Create a resizable editor sharing this plugin's parameters, meters, and editor state.
    fn editor(&mut self, _executor: AsyncExecutor<Self>) -> Option<Self::Editor> {
        create_egui_editor(
            self.editor_state.clone(),
            self.notifier.clone(),
            EguiNiceSettings::new().with_resize_hint({
                let (w, h) = crate::editor::minimum_size(ID);
                ResizeHint::resizable().with_min_logical_size(LogicalSize::new(w, h))
            }),
            NativeEditor::new(ID, self.params.clone(), self.meters.clone()),
        )
    }
    /// Prepare the plugin, rejecting non-finite rates and rates outside 8,000–384,000 Hz.
    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        config: &BufferConfig,
        _context: &mut impl ActivateContext<Self>,
    ) -> bool {
        if !config.sample_rate.is_finite() || config.sample_rate < 8000.0 || config.sample_rate > 384000.0 {
            return false;
        }
        self.offline = config.process_mode == ProcessMode::Offline;
        self.prepare(config.sample_rate);
        if let Some(disk) = self.sample.as_ref().and_then(|s| s.disk.as_ref()) {
            disk.set_offline(self.offline);
        }
        true
    }
    fn reset(&mut self) {
        self.clear();
    }
    /// Process a host block with per-sample smoothing and timestamped instrument events.
    /// Applies audition requests at the block boundary and uses 120 BPM when host tempo is absent.
    /// Requests an editor repaint and returns `KeepAlive` so tails and releasing voices continue.
    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let tempo = context.transport().tempo.unwrap_or(120.0);
        self.block_reduction = 0.0;
        self.refresh_sample();
        self.sync_preview(&self.params.values());
        let mut event = context.next_event();
        for (i, mut frame) in buffer.iter_samples().enumerate() {
            let p = self.params.next_values();
            while let Some(e) = event {
                if e.timing() > i as u32 {
                    break;
                }
                if ID < 2 {
                    self.event(e, &p);
                }
                event = context.next_event();
            }
            let key = aux.inputs.first_mut().and_then(|b| {
                let channels = b.as_slice();
                Some([*channels.first()?.get(i)?, *channels.get(1)?.get(i)?])
            });
            let input = [
                frame.get_mut(0).map_or(0.0, |v| *v),
                frame.get_mut(1).map_or(0.0, |v| *v),
            ];
            let out = self.frame(input, key, &p, tempo);
            self.block_reduction = self
                .block_reduction
                .max(self.effect.as_ref().map_or(0.0, |e| e.gain_reduction_db()));
            for (v, value) in frame.iter_mut().zip(out) {
                *v = value;
            }
        }
        let _ = context.try_send_event(NoteEvent::MidiSysEx {
            timing: 0,
            message: ClipSysEx::Telemetry(limeshot_dsp::PluginTelemetry {
                voices: self.voices[..self.active_end]
                    .iter()
                    .filter(|s| s.voice.is_active())
                    .count() as u32,
                reduction_db: if self.params.bypass.value() {
                    0.0
                } else {
                    self.block_reduction.clamp(0.0, 200.0)
                },
            }),
        });
        // The editor polls telemetry at its bounded repaint interval.
        // Reverb/delay and releasing voices must continue to receive silent blocks.
        ProcessStatus::KeepAlive
    }
}

impl<const ID: u8> Vst3Plugin for LimePlugin<ID> {
    const VST3_CLASS_ID: [u8; 16] = [
        b'L', b'I', b'M', b'E', b'N', b'A', b'T', b'I', b'V', b'E', 0, 0, 0, 0, 1, ID,
    ];
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = match ID {
        0 => &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth],
        1 => &[Vst3SubCategory::Instrument, Vst3SubCategory::Sampler],
        3 => &[Vst3SubCategory::Fx, Vst3SubCategory::Eq],
        4 => &[Vst3SubCategory::Fx, Vst3SubCategory::Filter],
        5 => &[Vst3SubCategory::Fx, Vst3SubCategory::Delay],
        6 => &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb],
        7..=9 => &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics],
        10 => &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion],
        11 => &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation],
        _ => &[Vst3SubCategory::Fx, Vst3SubCategory::Tools],
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::EmbeddedSample;
    use nice_plug::params::persist::PersistentField;

    fn on(id: i32, key: u8) -> NoteEvent<ClipSysEx> {
        NoteEvent::NoteOn {
            timing: 0,
            voice_id: VoiceID::ID(id),
            channel: Channel::Number(0),
            key: Key::Number(key),
            velocity: 0.8,
        }
    }
    fn off(id: i32, key: u8) -> NoteEvent<ClipSysEx> {
        NoteEvent::NoteOff {
            timing: 0,
            voice_id: VoiceID::ID(id),
            channel: Channel::Number(0),
            key: Key::Number(key),
            velocity: 0.0,
        }
    }
    fn cc(cc: u8, value: f32) -> NoteEvent<ClipSysEx> {
        NoteEvent::MidiCC {
            timing: 0,
            channel: 0,
            cc,
            value,
        }
    }

    #[test]
    fn disk_sampler_state_matches_embedded_pcm_across_blocks_and_seeks() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("disk.wav");
        let data: Vec<f32> = (0..180_000).flat_map(|i| [(i as f32 * 0.023).sin() * 0.4; 2]).collect();
        limeshot_dsp::write_wav(&source, 48_000, &data, limeshot_dsp::BitDepth::Float32).unwrap();
        let state = EmbeddedSample::load_mode(&source, true).unwrap();
        assert!(state.keep_on_disk && state.pcm.is_empty());
        for block in [1, 64, 333, 1024] {
            let mut disk = LimePlugin::<1>::default();
            disk.params.sample.set(state.clone());
            let fields = disk.params.serialize_fields();
            disk.params.deserialize_fields(&fields);
            disk.prepare(48_000.0);
            assert!(disk.sample.as_ref().unwrap().data.is_empty());
            let mut memory = LimePlugin::<1>::default();
            memory
                .params
                .sample
                .set(EmbeddedSample::load_mode(&source, false).unwrap());
            memory.prepare(48_000.0);
            let values = memory.params.values();
            disk.event(on(1, 60), &values);
            memory.event(on(1, 60), &values);
            // The first pages are primed: realtime processing neither waits nor allocates.
            nice_assert_no_alloc::assert_no_alloc(|| {
                for _ in 0..1024 {
                    assert_eq!(
                        disk.frame([0.0; 2], None, &values, 120.0),
                        memory.frame([0.0; 2], None, &values, 120.0)
                    );
                }
            });
            disk.sample.as_ref().unwrap().disk.as_ref().unwrap().set_offline(true);
            for slot in [&mut disk, &mut memory] {
                slot.voices[0].voice.skip(2.5, 48_000.0);
            }
            for start in (0..20_000).step_by(block) {
                for _ in start..(start + block).min(20_000) {
                    assert_eq!(
                        disk.frame([0.0; 2], None, &values, 120.0),
                        memory.frame([0.0; 2], None, &values, 120.0)
                    );
                }
            }
        }
    }

    fn parity<const ID: u8>() {
        for sr in [44100, 48000, 96000] {
            for block in [1, 64, 333, 1024] {
                let mut plugin = LimePlugin::<ID>::default();
                plugin.prepare(sr as f32);
                let p = plugin.params.values();
                let mut reference = build(
                    parameters::KINDS[(ID - 2) as usize],
                    sr,
                    &p[..parameters::specs(ID).len()],
                );
                let input: Vec<f32> = (0..4096)
                    .map(|i| if i < 2 { 0.9 } else { (i as f32 * 0.07).sin() * 0.2 })
                    .collect();
                let mut expected = input.clone();
                for chunk in expected.chunks_mut(block * 2) {
                    reference.process(chunk);
                }
                let actual: Vec<f32> = input
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .flat_map(|f| plugin.frame([f[0], f[1]], None, &p, 120.0))
                    .collect();
                assert!(
                    actual.iter().zip(&expected).all(|(a, b)| (a - b).abs() < 0.00001),
                    "ID {ID}, {sr}Hz, block {block}"
                );
            }
        }
    }
    #[test]
    fn exported_effects_share_the_daw_dsp_at_variable_block_sizes() {
        parity::<2>();
        parity::<3>();
        parity::<4>();
        parity::<5>();
        parity::<6>();
        parity::<7>();
        parity::<8>();
        parity::<9>();
        parity::<10>();
        parity::<11>();
    }

    #[test]
    fn limiter_holds_its_ceiling_and_reset_clears_meters() {
        let mut p = LimePlugin::<8>::default();
        p.prepare(48000.0);
        let values = p.params.values();
        let ceiling = db_to_gain(values[0]);
        for i in 0..48000 {
            let out = p.frame([if i % 2 == 0 { 10.0 } else { -10.0 }, 2.0], None, &values, 120.0);
            assert!(out.iter().all(|v| v.abs() <= ceiling + 0.00001));
        }
        assert!(Meters::read(&p.meters.reduction) > 10.0);
        p.clear();
        assert_eq!(Meters::read(&p.meters.output), 0.0);
    }
    #[test]
    fn instruments_keep_128_voices_and_steal_the_oldest() {
        let mut plugin = LimePlugin::<0>::default();
        plugin.prepare(48_000.0);
        let values = plugin.params.values();
        for id in 0..140 {
            plugin.event(on(id, 60), &values);
        }
        assert_eq!(plugin.voices.iter().filter(|v| v.voice.is_active()).count(), 128);
        assert!(plugin.voices.iter().all(|v| v.id.is_some_and(|id| id >= 12)));
        assert!(plugin.voices.iter().any(|v| v.id == Some(139)));
    }

    #[test]
    fn synth_release_sustain_repeated_keys_and_wildcard_choke() {
        let mut p = LimePlugin::<0>::default();
        p.prepare(48000.0);
        let values = p.params.values();
        p.event(on(1, 60), &values);
        p.event(on(2, 60), &values);
        p.event(cc(64, 1.0), &values);
        p.event(off(1, 60), &values);
        assert!(p.voices[0].sustained);
        assert!(!p.voices[1].sustained);
        for _ in 0..12000 {
            p.frame([0.0; 2], None, &values, 120.0);
        }
        assert!(p.voices[0].voice.is_active());
        p.event(cc(64, 0.0), &values);
        p.event(off(2, 60), &values);
        for _ in 0..12000 {
            p.frame([0.0; 2], None, &values, 120.0);
        }
        assert!(!p.voices.iter().any(|v| v.voice.is_active()));
        for i in 0..100 {
            p.event(on(i, 40 + (i % 40) as u8), &values);
        }
        assert_eq!(p.voices.iter().filter(|v| v.voice.is_active()).count(), 100);
        p.event(
            NoteEvent::Choke {
                timing: 0,
                voice_id: VoiceID::Wildcard,
                channel: Channel::Wildcard,
                key: Key::Wildcard,
            },
            &values,
        );
        assert!(!p.voices.iter().any(|v| v.voice.is_active()));
    }
    #[test]
    fn sampler_embeds_audio_and_recalls_without_the_original_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source.wav");
        let data =
            limeshot_dsp::SampleData::from_stereo(48000, (0..1000).flat_map(|i| [i as f32 / 1000.0; 2]).collect());
        limeshot_dsp::write_wav(&path, 48000, &data.data, limeshot_dsp::BitDepth::Float32).unwrap();
        let params = NativeParams::new(1);
        params.sample.set(EmbeddedSample::load(&path).unwrap());
        let saved = params.serialize_fields();
        std::fs::remove_file(path).unwrap();
        let mut plugin = LimePlugin::<1>::default();
        plugin.params.deserialize_fields(&saved);
        plugin.prepare(48000.0);
        let mut values = plugin.params.values();
        values[5] = 1.0;
        values[2] = 0.25;
        values[3] = 0.75;
        plugin.event(on(1, 60), &values);
        plugin.event(off(1, 60), &values);
        let first = plugin.frame([0.0; 2], None, &values, 120.0)[0];
        assert!((first - 0.749 * 0.8).abs() < 0.0001, "{first}");
        for _ in 0..1000 {
            plugin.frame([0.0; 2], None, &values, 120.0);
        }
        assert!(!plugin.voices.iter().any(|v| v.voice.is_active()));
        // Gated playback releases before the end of the sample.
        values[5] = 0.0;
        values[6] = 0.0;
        values[4] = 0.0;
        plugin.event(on(2, 60), &values);
        plugin.event(off(2, 60), &values);
        assert_eq!(plugin.frame([0.0; 2], None, &values, 120.0), [0.0; 2]);
    }
    #[test]
    fn external_sidechain_ducks_quiet_audio() {
        let mut plugin = LimePlugin::<7>::default();
        let mut params = NativeParams::new(7);
        params.external_key = BoolParam::new("External sidechain", true);
        plugin.params = Arc::new(params);
        plugin.prepare(48000.0);
        let values = plugin.params.values();
        let mut last = [0.0; 2];
        for _ in 0..24000 {
            last = plugin.frame([0.05; 2], Some([1.0; 2]), &values, 120.0);
        }
        assert!(last[0] < 0.02, "{last:?}");
        assert!(Meters::read(&plugin.meters.reduction) > 10.0);
    }
    #[test]
    fn sampler_takes_up_a_sample_loaded_while_active() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("late.wav");
        limeshot_dsp::write_wav(&path, 48000, &[0.5; 2000], limeshot_dsp::BitDepth::Float32).unwrap();
        let mut plugin = LimePlugin::<1>::default();
        plugin.prepare(48000.0);
        assert!(plugin.sample.is_none());
        // A host loading state (or a preset) after activation, as LimeShot does.
        plugin.params.sample.set(EmbeddedSample::load(&path).unwrap());
        plugin.refresh_sample();
        let values = plugin.params.values();
        plugin.event(on(1, 60), &values);
        assert!(plugin.frame([0.0; 2], None, &values, 120.0)[0] > 0.1);
    }
    #[test]
    fn sampler_replacement_retires_audio_without_locks_or_destruction() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("replace.wav");
        limeshot_dsp::write_wav(&path, 48000, &[0.5; 2000], limeshot_dsp::BitDepth::Float32).unwrap();
        let mut plugin = LimePlugin::<1>::default();
        plugin.params.sample.set(EmbeddedSample::load(&path).unwrap());
        plugin.prepare(48000.0);
        let old = Arc::downgrade(plugin.sample.as_ref().unwrap());
        let values = plugin.params.values();
        plugin.event(on(1, 60), &values);
        plugin.params.sample.set(EmbeddedSample::default());
        // Holding the persistence lock must not prevent a realtime handoff.
        let store = plugin.params.sample.clone();
        {
            let plugin = std::cell::RefCell::new(&mut plugin);
            store.map(|_| {
                nice_assert_no_alloc::assert_no_alloc(|| plugin.borrow_mut().refresh_sample());
            });
        }
        assert!(plugin.sample.is_none());
        assert!(!plugin.voices.iter().any(|s| s.voice.is_active()));
        assert!(
            old.upgrade().is_some(),
            "old audio is retained for control-thread destruction"
        );
        plugin.params.sample.set(EmbeddedSample::default());
        assert!(old.upgrade().is_none());
        // Queue saturation coalesces obsolete updates and keeps the most recent one.
        for index in 0..20 {
            plugin.params.sample.set(EmbeddedSample {
                name: format!("pending-{index}"),
                ..Default::default()
            });
        }
        plugin.params.sample.set(EmbeddedSample::load(&path).unwrap());
        for _ in 0..8 {
            nice_assert_no_alloc::assert_no_alloc(|| plugin.refresh_sample());
        }
        assert!(plugin.sample.is_some());
    }

    #[test]
    fn unchanged_sample_state_keeps_held_voices_and_decoded_audio() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("held.wav");
        limeshot_dsp::write_wav(&path, 48000, &[0.5; 2000], limeshot_dsp::BitDepth::Float32).unwrap();
        let mut plugin = LimePlugin::<1>::default();
        let sample = EmbeddedSample::load(&path).unwrap();
        plugin.params.sample.set(sample.clone());
        plugin.prepare(48000.0);
        let before = plugin.sample.clone().unwrap();
        let values = plugin.params.values();
        plugin.event(on(1, 60), &values);
        plugin.params.sample.set(sample);
        nice_assert_no_alloc::assert_no_alloc(|| plugin.refresh_sample());
        assert!(Arc::ptr_eq(&before, plugin.sample.as_ref().unwrap()));
        assert!(plugin.voices.iter().any(|v| v.voice.is_active()));
    }

    #[test]
    fn key_tracking_alone_activates_the_open_synth_filter() {
        let plugin = LimePlugin::<0>::default();
        let mut values = plugin.params.values();
        values[8] = 20_000.0;
        values[9] = 0.0;
        values[10] = 0.0;
        values[14] = 0.0;
        values[16] = 1.0;
        let state = plugin.instrument(&values);
        let voice = Voice::for_instrument(&state, 60, 36, 1.0, 48000.0).unwrap();
        assert!(matches!(
            voice.source,
            limeshot_dsp::voice::Source::Synth { mods: Some(_), .. }
        ));
    }

    #[test]
    fn ping_pong_disk_and_memory_match_without_realtime_allocation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("loop.wav");
        let data: Vec<f32> = (0..64).flat_map(|i| [i as f32 / 128.0, -(i as f32) / 128.0]).collect();
        limeshot_dsp::write_wav(&path, 48000, &data, limeshot_dsp::BitDepth::Float32).unwrap();
        for rate in [44100.0, 48000.0, 96000.0] {
            for block in [1, 64, 333, 1024] {
                for reverse in [0.0, 1.0] {
                    let mut disk = LimePlugin::<1>::default();
                    disk.params.sample.set(EmbeddedSample::load_mode(&path, true).unwrap());
                    disk.prepare(rate);
                    let mut memory = LimePlugin::<1>::default();
                    memory
                        .params
                        .sample
                        .set(EmbeddedSample::load_mode(&path, false).unwrap());
                    memory.prepare(rate);
                    let mut values = memory.params.values();
                    values[4] = 0.0;
                    values[5] = reverse;
                    values[6] = 0.0;
                    values[13] = 1.0;
                    values[14] = 0.25;
                    values[15] = 0.75;
                    values[16] = 1.0;
                    disk.event(on(1, 60), &values);
                    memory.event(on(1, 60), &values);
                    nice_assert_no_alloc::assert_no_alloc(|| {
                        for i in 0..4096 {
                            if i % block == 0 {
                                disk.update_voices(&values);
                                memory.update_voices(&values);
                            }
                            assert_eq!(
                                disk.frame([0.0; 2], None, &values, 120.0),
                                memory.frame([0.0; 2], None, &values, 120.0)
                            );
                        }
                    });
                    assert!(memory.voices[0].voice.is_active());
                    values[17] = 2.0;
                    memory.update_voices(&values);
                    assert!(
                        memory.voices[0].voice.filter.is_some(),
                        "HP must remain active at maximum cutoff"
                    );
                    values[16] = 0.0;
                    memory.update_voices(&values);
                    assert!(matches!(
                        memory.voices[0].voice.source,
                        limeshot_dsp::voice::Source::Sampler { ping_pong: false, .. }
                    ));
                }
            }
        }
    }

    #[test]
    fn held_sampler_notes_follow_loop_and_filter_edits() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("held.wav");
        limeshot_dsp::write_wav(&path, 48000, &[0.5; 2000], limeshot_dsp::BitDepth::Float32).unwrap();
        let mut plugin = LimePlugin::<1>::default();
        plugin.params.sample.set(EmbeddedSample::load(&path).unwrap());
        plugin.prepare(48000.0);
        let mut values = plugin.params.values();
        plugin.event(on(1, 60), &values);
        values[13] = 1.0;
        values[14] = 0.2;
        values[15] = 0.8;
        values[11] = 500.0;
        values[12] = 0.5;
        plugin.update_voices(&values);
        let voice = &mut plugin.voices.iter_mut().find(|s| s.voice.is_active()).unwrap().voice;
        assert!(voice.filter.is_some());
        assert!(
            matches!(voice.source, limeshot_dsp::voice::Source::Sampler { looping: Some((a,b)), .. } if (a-200.0).abs()<0.01 && (b-800.0).abs()<0.01)
        );
        for _ in 0..3000 {
            voice.next_frame();
        }
        assert!(voice.is_active(), "a held note keeps looping past the source end");
        values[13] = 0.0;
        values[11] = 20_000.0;
        values[12] = 0.0;
        plugin.update_voices(&values);
        let voice = &plugin.voices.iter().find(|s| s.voice.is_active()).unwrap().voice;
        assert!(voice.filter.is_none());
        assert!(matches!(
            voice.source,
            limeshot_dsp::voice::Source::Sampler { looping: None, .. }
        ));
    }

    #[test]
    fn tempo_sync_moves_delay_echo_and_bypass_passes_exact_audio() {
        let mut slow = LimePlugin::<5>::default();
        slow.prepare(48000.0);
        let mut fast = LimePlugin::<5>::default();
        fast.prepare(48000.0);
        let mut values = slow.params.values();
        values[1] = 0.0;
        values[2] = 0.0;
        values[4] = 1.0;
        values[5] = 5.0;
        let mut slow_on = None;
        let mut fast_on = None;
        for i in 0..50000 {
            let input = if i == 0 { [1.0; 2] } else { [0.0; 2] };
            let a = slow.frame(input, None, &values, 60.0);
            let b = fast.frame(input, None, &values, 120.0);
            if a[0].abs() > 0.001 && slow_on.is_none() {
                slow_on = Some(i);
            }
            if b[0].abs() > 0.001 && fast_on.is_none() {
                fast_on = Some(i);
            }
        }
        assert!(slow_on.is_some_and(|n: i32| (n - 48000).abs() <= 4));
        assert!(fast_on.is_some_and(|n: i32| (n - 24000).abs() <= 4));
        let mut plugin = LimePlugin::<8>::default();
        let mut params = NativeParams::new(8);
        params.bypass = BoolParam::new("Bypass", true).make_bypass();
        plugin.params = Arc::new(params);
        plugin.prepare(48000.0);
        let values = plugin.params.values();
        assert_eq!(plugin.frame([2.0, -3.0], None, &values, 120.0), [2.0, -3.0]);
    }
    #[test]
    fn held_synth_notes_follow_filter_and_pitch_automation() {
        let mut p = LimePlugin::<0>::default();
        p.prepare(48000.0);
        let mut values = p.params.values();
        p.event(on(1, 69), &values);
        for _ in 0..12000 {
            p.frame([0.0; 2], None, &values, 120.0);
        }
        values[8] = 50.0;
        values[9] = 0.3;
        values[3] = -12.0;
        values[1] = 1.0;
        p.frame([0.0; 2], None, &values, 120.0);
        assert!(p.voices[0].voice.is_active());
        let peak = (0..12000)
            .map(|_| p.frame([0.0; 2], None, &values, 120.0)[0].abs())
            .fold(0.0_f32, f32::max);
        assert!(peak < 0.15, "{peak}");
        p.event(
            NoteEvent::MidiPitchBend {
                timing: 0,
                channel: 0,
                value: 1.0,
            },
            &values,
        );
        assert_eq!(p.bend[0], 2.0);
    }
    #[test]
    fn synth_filter_type_and_key_tracking_reach_the_voice() {
        // Peak of a sine note after its attack, with `set` applied to the default controls.
        let peak = |key: u8, set: &dyn Fn(&mut [f32; MAX_CONTROLS])| {
            let mut p = LimePlugin::<0>::default();
            p.prepare(48000.0);
            let mut values = p.params.values();
            values[0] = 0.0;
            set(&mut values);
            p.event(on(1, key), &values);
            for _ in 0..4800 {
                p.frame([0.0; 2], None, &values, 120.0);
            }
            (0..4800)
                .map(|_| p.frame([0.0; 2], None, &values, 120.0)[0].abs())
                .fold(0.0_f32, f32::max)
        };
        // High-pass at 2 kHz removes a 110 Hz note that the open filter passes.
        let open = peak(45, &|_| {});
        let high = peak(45, &|v| {
            v[8] = 2000.0;
            v[15] = 2.0;
        });
        assert!(high < 0.1 * open, "{high} vs {open}");
        // A low-pass at 200 Hz silences C7 unless the cutoff follows the key.
        let fixed = peak(96, &|v| v[8] = 200.0);
        let tracked = peak(96, &|v| {
            v[8] = 200.0;
            v[16] = 1.0;
        });
        assert!(tracked > 10.0 * fixed, "{tracked} vs {fixed}");
    }
    #[test]
    fn editor_audition_starts_and_releases_at_block_boundaries() {
        for block in [1, 64, 333, 1024] {
            let mut plugin = LimePlugin::<1>::default();
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/samples/Lime Snare.wav");
            plugin.params.sample.set(EmbeddedSample::load(&path).unwrap());
            plugin.prepare(48000.0);
            let values = plugin.params.values();
            assert_eq!(plugin.frame([0.0; 2], None, &values, 120.0), [0.0; 2]);
            plugin.meters.preview_note.store(61, Ordering::Relaxed);
            plugin.sync_preview(&values);
            assert_eq!(plugin.voices.iter().filter(|s| s.voice.is_active()).count(), 1);
            let mut peak = 0.0_f32;
            for _ in 0..block * 64 {
                peak = peak.max(plugin.frame([0.0; 2], None, &values, 120.0)[0].abs());
            }
            assert!(peak > 0.00001, "block {block}");
            plugin.meters.preview_note.store(0, Ordering::Relaxed);
            plugin.sync_preview(&values);
            // A one-shot preview finishes the WAV after mouse-up, matching the sampler setting.
            for _ in 0..48000 {
                plugin.frame([0.0; 2], None, &values, 120.0);
            }
            assert!(!plugin.voices.iter().any(|s| s.voice.is_active()));
            plugin.meters.preview_note.store(128, Ordering::Relaxed);
            plugin.sync_preview(&values);
            assert!(plugin.voices.iter().any(|s| s.voice.key == 127 && s.voice.is_active()));
            plugin.clear();
            assert_eq!(plugin.meters.preview_note.load(Ordering::Relaxed), 0);
        }
    }
}
