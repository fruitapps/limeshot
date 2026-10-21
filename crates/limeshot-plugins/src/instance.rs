//! A VST3 plugin instance, split into its control-thread and audio-thread halves.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use vst3::Steinberg::Vst::*;
use vst3::Steinberg::*;
use vst3::{ComPtr, ComWrapper};

use crate::host::{ComponentHandler, EventList, HostApp, MemoryStream, ParamChanges};
use crate::util::from_wstr;
use crate::{PluginError, Vst3Editor, Vst3Module};
pub use limeshot_dsp::{NoteEvent, NoteEventKind, Transport};

/// Parameter changes queued from the control thread per block, and changed parameters per block.
const PARAM_QUEUES: usize = 64;
/// Parameter changes the control thread can queue before the audio thread takes them.
const PARAM_RING: usize = 1024;
/// Note events per block.
const MAX_EVENTS: usize = 512;
/// Channels kept per bus (more than any stereo plugin uses; surround buses get silence).
const MAX_BUS_CHANNELS: usize = 8;

/// One parameter, as the plugin's controller describes it. Values are normalised (0–1).
#[derive(Debug, Clone, PartialEq)]
pub struct ParamInfo {
    pub id: u32,
    pub title: String,
    pub units: String,
    /// 0 for a continuous parameter, otherwise the number of steps (1 for an on/off switch).
    pub step_count: i32,
    pub default: f64,
    pub flags: i32,
}

impl ParamInfo {
    pub fn automatable(&self) -> bool {
        self.flags & ParameterInfo_::ParameterFlags_::kCanAutomate != 0
    }

    pub fn read_only(&self) -> bool {
        self.flags & ParameterInfo_::ParameterFlags_::kIsReadOnly != 0
    }

    pub fn is_bypass(&self) -> bool {
        self.flags & ParameterInfo_::ParameterFlags_::kIsBypass != 0
    }
}

/// What both halves share; tears the plugin down when the last half goes.
struct Core {
    // Field order is teardown order after `Drop::drop`: interfaces first, the module last.
    connection: Option<(ComPtr<IConnectionPoint>, ComPtr<IConnectionPoint>)>,
    controller: Option<ComPtr<IEditController>>,
    separate_controller: bool,
    processor: ComPtr<IAudioProcessor>,
    component: ComPtr<IComponent>,
    handler: ComWrapper<ComponentHandler>,
    _host: ComWrapper<HostApp>,
    active: AtomicBool,
    _module: Arc<Vst3Module>,
}

// SAFETY: VST3 objects may be used from several threads under the API's threading rules, which
// LimeShot follows: `Vst3Processor` calls only processing functions (audio thread) and
// `Vst3Instance` everything else (control thread). The reference counts are thread-safe.
unsafe impl Send for Core {}
unsafe impl Sync for Core {}

impl Drop for Core {
    fn drop(&mut self) {
        // SAFETY: teardown in the order the VST3 documentation gives; every call is made on a
        // live object. This runs on the control thread (the audio thread hands processors back).
        unsafe {
            if self.active.swap(false, Ordering::AcqRel) {
                self.component.setActive(0);
            }
            if let Some((a, b)) = &self.connection {
                a.disconnect(b.as_ptr());
                b.disconnect(a.as_ptr());
            }
            if let Some(c) = &self.controller {
                c.setComponentHandler(std::ptr::null_mut());
                if self.separate_controller {
                    c.terminate();
                }
            }
            self.component.terminate();
        }
    }
}

/// The control-thread side of a plugin: parameters, state, latency.
pub struct Vst3Instance {
    core: Arc<Core>,
    params: Vec<ParamInfo>,
    to_audio: rtrb::Producer<(u32, f64)>,
}

/// The audio-thread side of a plugin: processing audio and notes. Hand it back to the control
/// thread to drop it.
pub struct Vst3Processor {
    core: Arc<Core>,
    max_block: usize,
    inputs: Vec<Bus>,
    outputs: Vec<Bus>,
    in_bufs: Vec<AudioBusBuffers>,
    out_bufs: Vec<AudioBusBuffers>,
    has_event_input: bool,
    events: ComWrapper<EventList>,
    params_in: ComWrapper<ParamChanges>,
    params_out: ComWrapper<ParamChanges>,
    context: ProcessContext,
    from_control: rtrb::Consumer<(u32, f64)>,
    processing: bool,
    transport: Transport,
    sample_rate: f64,
}

// SAFETY: the raw pointers inside are into buffers this struct owns; see `Core` for the plugin.
unsafe impl Send for Vst3Processor {}

struct Bus {
    channels: usize,
    data: Vec<f32>,
    ptrs: Vec<*mut f32>,
}

impl Bus {
    fn new(channels: usize, max_block: usize) -> Self {
        let channels = channels.min(MAX_BUS_CHANNELS);
        Self {
            channels,
            data: vec![0.0; channels * max_block],
            ptrs: vec![std::ptr::null_mut(); channels],
        }
    }

    fn channel(&mut self, c: usize, max_block: usize, frames: usize) -> &mut [f32] {
        &mut self.data[c * max_block..c * max_block + frames]
    }

    /// Point `ptrs` at the channel buffers and describe them for `ProcessData`.
    fn buffers(&mut self, max_block: usize) -> AudioBusBuffers {
        for (c, p) in self.ptrs.iter_mut().enumerate() {
            *p = self.data[c * max_block..].as_mut_ptr();
        }
        AudioBusBuffers {
            numChannels: self.channels as i32,
            silenceFlags: 0,
            __field0: AudioBusBuffers__type0 {
                channelBuffers32: self.ptrs.as_mut_ptr(),
            },
        }
    }
}

fn channel_count(arr: SpeakerArrangement) -> usize {
    arr.count_ones() as usize
}

impl Vst3Instance {
    /// Create, initialise and activate class `class_id` of `module` for stereo processing.
    pub fn new(
        module: &Arc<Vst3Module>,
        class_id: &str,
        sample_rate: f64,
        max_block: usize,
    ) -> Result<(Vst3Instance, Vst3Processor), PluginError> {
        let max_block = max_block.max(1);
        let component: ComPtr<IComponent> = module
            .create(class_id)
            .ok_or_else(|| PluginError::NoClass(class_id.to_owned()))?;
        let host = ComWrapper::new(HostApp);
        let host_ptr = host
            .as_com_ref::<IHostApplication>()
            .map(|r| r.as_ptr() as *mut FUnknown)
            .ok_or(PluginError::Call("create the host context"))?;
        // SAFETY: every call below is a documented VST3 setup call on a live object, with
        // pointers to stack or owned memory of the expected type.
        unsafe {
            if component.initialize(host_ptr) != kResultOk {
                return Err(PluginError::Call("initialize"));
            }
            let mut separate_controller = false;
            let mut controller = component.cast::<IEditController>();
            if controller.is_none() {
                let mut cid: TUID = [0; 16];
                if component.getControllerClassId(&mut cid) == kResultOk
                    && let Some(c) = module.create_tuid::<IEditController>(&cid)
                    && c.initialize(host_ptr) == kResultOk
                {
                    separate_controller = true;
                    controller = Some(c);
                }
            }
            let Some(processor) = component.cast::<IAudioProcessor>() else {
                component.terminate();
                return Err(PluginError::Unsupported("audio processing"));
            };
            let handler = ComWrapper::new(ComponentHandler::default());
            let mut core = Core {
                connection: None,
                controller,
                separate_controller,
                processor,
                component,
                handler,
                _host: host,
                active: AtomicBool::new(false),
                _module: module.clone(),
            };
            if let Some(c) = &core.controller {
                if let Some(h) = core.handler.as_com_ref::<IComponentHandler>() {
                    c.setComponentHandler(h.as_ptr());
                }
                if core.separate_controller
                    && let (Some(a), Some(b)) =
                        (core.component.cast::<IConnectionPoint>(), c.cast::<IConnectionPoint>())
                {
                    a.connect(b.as_ptr());
                    b.connect(a.as_ptr());
                    core.connection = Some((a, b));
                }
                // The controller learns the processor's state, as after loading a preset.
                let stream = ComWrapper::new(MemoryStream::default());
                if let Some(s) = stream.as_com_ref::<IBStream>()
                    && core.component.getState(s.as_ptr()) == kResultOk
                {
                    s.seek(0, IBStream_::IStreamSeekMode_::kIBSeekSet as i32, std::ptr::null_mut());
                    c.setComponentState(s.as_ptr());
                }
            }
            if core
                .processor
                .canProcessSampleSize(SymbolicSampleSizes_::kSample32 as i32)
                != kResultOk
            {
                return Err(PluginError::Unsupported("32-bit float audio"));
            }

            let (inputs, outputs) = Self::setup_buses(&core, max_block);
            let has_event_input = core
                .component
                .getBusCount(MediaTypes_::kEvent as i32, BusDirections_::kInput as i32)
                > 0;
            if has_event_input {
                core.component
                    .activateBus(MediaTypes_::kEvent as i32, BusDirections_::kInput as i32, 0, 1);
            }
            let mut setup = ProcessSetup {
                processMode: ProcessModes_::kRealtime as i32,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32 as i32,
                maxSamplesPerBlock: max_block as i32,
                sampleRate: sample_rate,
            };
            if core.processor.setupProcessing(&mut setup) != kResultOk {
                return Err(PluginError::Call("set up processing"));
            }
            if core.component.setActive(1) != kResultOk {
                return Err(PluginError::Call("activate"));
            }
            core.active.store(true, Ordering::Release);

            let params = Self::read_params(&core);
            let core = Arc::new(core);
            let (to_audio, from_control) = rtrb::RingBuffer::new(PARAM_RING);
            let in_bufs = Vec::with_capacity(inputs.len());
            let out_bufs = Vec::with_capacity(outputs.len());
            let processor = Vst3Processor {
                core: core.clone(),
                max_block,
                inputs,
                outputs,
                in_bufs,
                out_bufs,
                has_event_input,
                events: ComWrapper::new(EventList::new(MAX_EVENTS)),
                params_in: ComWrapper::new(ParamChanges::new(PARAM_QUEUES)),
                params_out: ComWrapper::new(ParamChanges::new(PARAM_QUEUES)),
                context: std::mem::zeroed(),
                from_control,
                processing: false,
                transport: Transport::default(),
                sample_rate,
            };
            Ok((Vst3Instance { core, params, to_audio }, processor))
        }
    }

    /// Ask for stereo main buses, switch auxiliary buses off, and size a buffer per bus.
    ///
    /// # Safety
    /// `core` must hold an initialised, inactive component.
    unsafe fn setup_buses(core: &Core, max_block: usize) -> (Vec<Bus>, Vec<Bus>) {
        let audio = MediaTypes_::kAudio as i32;
        let (dir_in, dir_out) = (BusDirections_::kInput as i32, BusDirections_::kOutput as i32);
        // SAFETY: calls on the live component and processor (see the function docs).
        unsafe {
            let n_in = core.component.getBusCount(audio, dir_in).max(0) as usize;
            let n_out = core.component.getBusCount(audio, dir_out).max(0) as usize;
            let mut arr_in = vec![SpeakerArr::kStereo; n_in];
            let mut arr_out = vec![SpeakerArr::kStereo; n_out];
            for (dir, arr) in [(dir_in, &mut arr_in), (dir_out, &mut arr_out)] {
                for (i, a) in arr.iter_mut().enumerate().skip(1) {
                    let mut cur: SpeakerArrangement = 0;
                    if core.processor.getBusArrangement(dir, i as i32, &mut cur) == kResultOk {
                        *a = cur;
                    }
                }
            }
            // A plugin may refuse stereo; it then keeps its own layout, read back below.
            core.processor.setBusArrangements(
                if n_in > 0 {
                    arr_in.as_mut_ptr()
                } else {
                    std::ptr::null_mut()
                },
                n_in as i32,
                if n_out > 0 {
                    arr_out.as_mut_ptr()
                } else {
                    std::ptr::null_mut()
                },
                n_out as i32,
            );
            let buses = |dir: i32, n: usize| -> Vec<Bus> {
                (0..n)
                    .map(|i| {
                        core.component.activateBus(audio, dir, i as i32, (i == 0) as u8);
                        let mut a: SpeakerArrangement = SpeakerArr::kStereo;
                        core.processor.getBusArrangement(dir, i as i32, &mut a);
                        Bus::new(channel_count(a), max_block)
                    })
                    .collect()
            };
            (buses(dir_in, n_in), buses(dir_out, n_out))
        }
    }

    /// # Safety
    /// `core` must hold an initialised plugin.
    unsafe fn read_params(core: &Core) -> Vec<ParamInfo> {
        let Some(c) = &core.controller else {
            return Vec::new();
        };
        // SAFETY: controller queries into a zeroed struct of the documented type.
        unsafe {
            (0..c.getParameterCount().max(0))
                .filter_map(|i| {
                    let mut p: ParameterInfo = std::mem::zeroed();
                    (c.getParameterInfo(i, &mut p) == kResultOk).then(|| ParamInfo {
                        id: p.id,
                        title: from_wstr(&p.title),
                        units: from_wstr(&p.units),
                        step_count: p.stepCount,
                        default: p.defaultNormalizedValue,
                        flags: p.flags,
                    })
                })
                .collect()
        }
    }

    pub fn params(&self) -> &[ParamInfo] {
        &self.params
    }

    /// The current normalised value of parameter `id`.
    pub fn param(&self, id: u32) -> f64 {
        match &self.core.controller {
            // SAFETY: a plain query on the live controller.
            Some(c) => unsafe { c.getParamNormalized(id) },
            None => 0.0,
        }
    }

    /// Set parameter `id` (normalised) in the controller and, at the next block, the processor.
    pub fn set_param(&mut self, id: u32, value: f64) {
        let value = if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 };
        if let Some(c) = &self.core.controller {
            // SAFETY: a plain call on the live controller.
            unsafe { c.setParamNormalized(id, value) };
        }
        let _ = self.to_audio.push((id, value));
    }

    /// How the plugin shows `value` (normalised) of parameter `id`, such as "-6.0 dB".
    pub fn param_text(&self, id: u32, value: f64) -> String {
        let Some(c) = &self.core.controller else {
            return String::new();
        };
        let mut s: String128 = [0; 128];
        // SAFETY: the plugin writes at most 128 characters into `s`.
        if unsafe { c.getParamStringByValue(id, value, &mut s) } == kResultOk {
            from_wstr(&s)
        } else {
            format!("{value:.3}")
        }
    }

    /// Changes made in the plugin's own editor since the last call. They are already on their
    /// way to the processor; the caller records them (undo, automation).
    pub fn take_editor_edits(&mut self) -> Vec<(u32, f64)> {
        let edits = match self.core.handler.edits.lock() {
            Ok(mut e) => std::mem::take(&mut *e),
            Err(_) => Vec::new(),
        };
        for &(id, v) in &edits {
            let _ = self.to_audio.push((id, v));
        }
        edits
    }

    /// Whether the plugin reported changed parameter values or settings (`restartComponent`)
    /// since the last call, as plugins do after loading one of their own presets.
    pub fn take_restart(&mut self) -> bool {
        self.core.handler.restart_flags.swap(0, Ordering::Relaxed) != 0
    }

    /// The plugin's latency in samples, for delay compensation.
    pub fn latency_samples(&self) -> u32 {
        // SAFETY: a plain query on the live processor.
        unsafe { self.core.processor.getLatencySamples() }
    }

    /// The processor's and the controller's state, as one blob for the project file.
    pub fn state(&self) -> Result<Vec<u8>, PluginError> {
        let read = |f: &dyn Fn(*mut IBStream) -> tresult| -> Result<Vec<u8>, PluginError> {
            let stream = ComWrapper::new(MemoryStream::default());
            let s = stream.as_com_ref::<IBStream>().ok_or(PluginError::State)?;
            match f(s.as_ptr()) {
                r if r == kResultOk => Ok(stream.take()),
                r if r == kNotImplemented => Ok(Vec::new()),
                _ => Err(PluginError::Call("save its state")),
            }
        };
        // SAFETY: state calls on live objects with a live stream.
        let component = read(&|s| unsafe { self.core.component.getState(s) })?;
        let controller = match (&self.core.controller, self.core.separate_controller) {
            // SAFETY: as above.
            (Some(c), true) => read(&|s| unsafe { c.getState(s) }).unwrap_or_default(),
            _ => Vec::new(),
        };
        let mut out = Vec::with_capacity(12 + component.len() + controller.len());
        out.extend_from_slice(STATE_MAGIC);
        out.extend_from_slice(&(component.len() as u32).to_le_bytes());
        out.extend_from_slice(&component);
        out.extend_from_slice(&(controller.len() as u32).to_le_bytes());
        out.extend_from_slice(&controller);
        Ok(out)
    }

    /// Restore a blob from [`state`](Self::state).
    pub fn set_state(&mut self, blob: &[u8]) -> Result<(), PluginError> {
        let (component, controller) = split_state(blob).ok_or(PluginError::State)?;
        let stream = |bytes: &[u8]| ComWrapper::new(MemoryStream::with_data(bytes.to_vec()));
        // SAFETY: state calls on live objects with live streams.
        unsafe {
            let s = stream(component);
            let p = s.as_com_ref::<IBStream>().ok_or(PluginError::State)?;
            if self.core.component.setState(p.as_ptr()) != kResultOk && !component.is_empty() {
                return Err(PluginError::Call("load its state"));
            }
            if let Some(c) = &self.core.controller {
                let s = stream(component);
                if let Some(p) = s.as_com_ref::<IBStream>() {
                    c.setComponentState(p.as_ptr());
                }
                if self.core.separate_controller && !controller.is_empty() {
                    let s = stream(controller);
                    if let Some(p) = s.as_com_ref::<IBStream>() {
                        c.setState(p.as_ptr());
                    }
                }
            }
        }
        Ok(())
    }

    /// The plugin's own editor, if it has one for this platform. Main thread only.
    pub fn create_editor(&self) -> Option<Vst3Editor> {
        let c = self.core.controller.as_ref()?;
        // SAFETY: `createView` on the live controller; the returned view carries one reference,
        // which the `ComPtr` takes over. The editor keeps the plugin alive through `core`.
        unsafe {
            let view = ComPtr::from_raw(c.createView(ViewType::kEditor))?;
            Vst3Editor::new(view, self.core.clone(), self.core._module.is_builtin())
        }
    }
}

const STATE_MAGIC: &[u8; 4] = b"LSV3";

fn split_state(blob: &[u8]) -> Option<(&[u8], &[u8])> {
    let rest = blob.strip_prefix(STATE_MAGIC)?;
    let take = |b: &[u8]| -> Option<(usize, usize)> {
        let n = u32::from_le_bytes(b.get(..4)?.try_into().ok()?) as usize;
        (b.len() >= 4 + n).then_some((4, 4 + n))
    };
    let (a0, a1) = take(rest)?;
    let component = &rest[a0..a1];
    let rest = &rest[a1..];
    let (b0, b1) = take(rest)?;
    Some((component, &rest[b0..b1]))
}

impl Vst3Processor {
    /// Process `buf` (interleaved stereo) in place: the input is read from it (if the plugin has
    /// an audio input) and the output written over it. `notes` offsets are frames into `buf`.
    /// Does not allocate or lock.
    pub fn process(&mut self, buf: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        self.transport = *transport;
        let frames = buf.len() / 2;
        let mut start = 0;
        while start < frames {
            let n = (frames - start).min(self.max_block);
            self.process_chunk(&mut buf[2 * start..2 * (start + n)], start as u32, notes);
            start += n;
            self.transport.sample_pos += n as i64;
            self.transport.ppq_pos += n as f64 * self.transport.tempo / (60.0 * self.sample_rate);
        }
    }

    fn process_chunk(&mut self, buf: &mut [f32], first: u32, notes: &[NoteEvent]) {
        let n = buf.len() / 2;
        let mb = self.max_block;
        if !self.processing {
            // SAFETY: the audio-thread start call on the live processor.
            unsafe { self.core.processor.setProcessing(1) };
            self.processing = true;
        }

        self.params_in.clear();
        self.params_out.clear();
        while let Ok((id, v)) = self.from_control.pop() {
            self.params_in.add(id, 0, v);
        }
        self.events.clear();
        if self.has_event_input {
            for e in notes {
                if e.offset < first || e.offset >= first + n as u32 {
                    continue;
                }
                self.events.push(to_event(e, e.offset - first));
            }
        }

        for (b, bus) in self.inputs.iter_mut().enumerate() {
            let channels = bus.channels;
            for c in 0..channels {
                let ch = bus.channel(c, mb, n);
                match (b, channels) {
                    // A mono main input gets the sum of both sides.
                    (0, 1) => {
                        for (i, s) in ch.iter_mut().enumerate() {
                            *s = 0.5 * (buf[2 * i] + buf[2 * i + 1]);
                        }
                    }
                    (0, 2) => {
                        for (i, s) in ch.iter_mut().enumerate() {
                            *s = buf[2 * i + c];
                        }
                    }
                    _ => ch.fill(0.0),
                }
            }
        }
        for bus in &mut self.outputs {
            bus.data.fill(0.0);
        }
        self.in_bufs.clear();
        for bus in &mut self.inputs {
            self.in_bufs.push(bus.buffers(mb));
        }
        self.out_bufs.clear();
        for bus in &mut self.outputs {
            self.out_bufs.push(bus.buffers(mb));
        }

        self.fill_context();
        let ptr_or_null = |v: &mut Vec<AudioBusBuffers>| {
            if v.is_empty() {
                std::ptr::null_mut()
            } else {
                v.as_mut_ptr()
            }
        };
        let mut data = ProcessData {
            processMode: ProcessModes_::kRealtime as i32,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as i32,
            numSamples: n as i32,
            numInputs: self.in_bufs.len() as i32,
            numOutputs: self.out_bufs.len() as i32,
            inputs: ptr_or_null(&mut self.in_bufs),
            outputs: ptr_or_null(&mut self.out_bufs),
            inputParameterChanges: com_ptr(&self.params_in),
            outputParameterChanges: com_ptr(&self.params_out),
            inputEvents: if self.has_event_input {
                com_ptr(&self.events)
            } else {
                std::ptr::null_mut()
            },
            outputEvents: std::ptr::null_mut(),
            processContext: &mut self.context,
        };
        // SAFETY: every pointer in `data` is into buffers this struct owns, sized for
        // `max_block` frames, and stays valid for the call.
        unsafe { self.core.processor.process(&mut data) };

        match self.outputs.first_mut() {
            Some(bus) if bus.channels > 0 => {
                let (l, r) = (0, if bus.channels > 1 { 1 } else { 0 });
                for i in 0..n {
                    buf[2 * i] = bus.data[l * mb + i];
                    buf[2 * i + 1] = bus.data[r * mb + i];
                }
            }
            _ => buf.fill(0.0),
        }
    }

    fn fill_context(&mut self) {
        use ProcessContext_::StatesAndFlags_::*;
        let t = &self.transport;
        let c = &mut self.context;
        c.state = (kTempoValid | kTimeSigValid | kProjectTimeMusicValid | kContTimeValid) as u32;
        if t.playing {
            c.state |= kPlaying as u32;
        }
        c.sampleRate = self.sample_rate;
        c.projectTimeSamples = t.sample_pos;
        c.continousTimeSamples = t.sample_pos;
        c.projectTimeMusic = t.ppq_pos;
        let bar = (t.time_sig.0.max(1) as f64) * 4.0 / (t.time_sig.1.max(1) as f64);
        c.barPositionMusic = (t.ppq_pos / bar).floor() * bar;
        c.tempo = t.tempo;
        c.timeSigNumerator = t.time_sig.0;
        c.timeSigDenominator = t.time_sig.1;
    }

    /// Restart processing (clears tails), from the audio thread.
    pub fn restart(&mut self) {
        if self.processing {
            // SAFETY: audio-thread processing calls on the live processor.
            unsafe {
                self.core.processor.setProcessing(0);
                self.core.processor.setProcessing(1);
            }
        }
    }
}

impl Drop for Vst3Processor {
    fn drop(&mut self) {
        if self.processing {
            // SAFETY: the stop call that pairs with the first `process`.
            unsafe { self.core.processor.setProcessing(0) };
        }
    }
}

impl limeshot_dsp::effects::Effect for Vst3Processor {
    /// Plugin parameters are not LimeShot effect parameters; they reach the processor through
    /// [`Vst3Instance::set_param`].
    fn set_params(&mut self, _params: &[f32]) {}

    fn process(&mut self, buf: &mut [f32]) {
        let t = self.transport;
        Vst3Processor::process(self, buf, &[], &t);
    }

    fn reset(&mut self) {
        self.restart();
    }

    fn set_transport(&mut self, transport: &Transport) {
        self.transport = *transport;
    }
}

impl limeshot_dsp::Instrument for Vst3Processor {
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        out.fill(0.0);
        Vst3Processor::process(self, out, notes, transport);
    }

    fn reset(&mut self) {
        self.restart();
    }
}

fn com_ptr<C: vst3::Class, I: vst3::Interface>(w: &ComWrapper<C>) -> *mut I {
    w.as_com_ref::<I>().map(|r| r.as_ptr()).unwrap_or(std::ptr::null_mut())
}

fn to_event(e: &NoteEvent, offset: u32) -> Event {
    // SAFETY: `Event` is plain old data; zero is a valid value for every field.
    let mut ev: Event = unsafe { std::mem::zeroed() };
    ev.sampleOffset = offset as i32;
    ev.flags = Event_::EventFlags_::kIsLive as u16;
    match e.kind {
        NoteEventKind::On { key, velocity, id } => {
            ev.r#type = Event_::EventTypes_::kNoteOnEvent as u16;
            ev.__field0.noteOn = NoteOnEvent {
                channel: 0,
                pitch: key as i16,
                tuning: 0.0,
                velocity: velocity.clamp(0.0, 1.0),
                length: 0,
                noteId: id,
            };
        }
        NoteEventKind::Off { key, id } => {
            ev.r#type = Event_::EventTypes_::kNoteOffEvent as u16;
            ev.__field0.noteOff = NoteOffEvent {
                channel: 0,
                pitch: key as i16,
                velocity: 0.0,
                noteId: id,
                tuning: 0.0,
            };
        }
    }
    ev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_blob_round_trips_and_rejects_garbage() {
        let mut blob = STATE_MAGIC.to_vec();
        blob.extend_from_slice(&3u32.to_le_bytes());
        blob.extend_from_slice(b"abc");
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(b"z");
        assert_eq!(split_state(&blob), Some((&b"abc"[..], &b"z"[..])));
        assert_eq!(split_state(b"LSV3\xff\xff\xff\xff"), None);
        assert_eq!(split_state(b"nope"), None);
    }
}
