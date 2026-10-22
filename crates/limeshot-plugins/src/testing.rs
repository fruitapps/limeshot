//! Small VST3 plugins of LimeShot's own, compiled into the program, so hosting can be tested
//! anywhere without installed plugins:
//!
//! * **Lime Test Gain** (effect): stereo in and out, one parameter (linear gain, 0–1), with a
//!   separate controller class, as most commercial plugins have, and a minimal editor.
//! * **Lime Test Tone** (instrument): a sine for the last held key, one parameter (level), with
//!   processor and controller in one object, no editor, and three factory programs (a
//!   program-change parameter that sets the level).

use std::cell::Cell;
use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use vst3::Steinberg::Vst::*;
use vst3::Steinberg::*;
use vst3::{Class, ComRef, ComWrapper, uid};

use crate::util::{copy_cstr, copy_wstr, tuid_to_hex};
use crate::{PluginError, Vst3Module};

const GAIN_CID: TUID = uid(0x4C494D45, 0x54474149, 0x4E000000, 0x00000001);
const GAIN_CTRL_CID: TUID = uid(0x4C494D45, 0x54474149, 0x4E000000, 0x00000002);
const TONE_CID: TUID = uid(0x4C494D45, 0x54544F4E, 0x45000000, 0x00000001);

/// The path that stands for these plugins in a project or the plugin database.
pub const BUILTIN_PATH: &str = "builtin:limeshot-test";

/// The class id of Lime Test Gain.
pub fn gain_class() -> String {
    tuid_to_hex(&GAIN_CID)
}

/// The class id of Lime Test Tone.
pub fn tone_class() -> String {
    tuid_to_hex(&TONE_CID)
}

/// A module holding both test plugins.
pub fn module() -> Result<Arc<Vst3Module>, PluginError> {
    Vst3Module::from_factory(get_factory)
}

unsafe extern "system" fn get_factory() -> *mut IPluginFactory {
    ComWrapper::new(Factory)
        .to_com_ptr::<IPluginFactory>()
        .map(|p| p.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// A normalised parameter value shared between threads.
struct Value(AtomicU64);

impl Value {
    fn new(v: f64) -> Self {
        Self(AtomicU64::new(v.to_bits()))
    }
    fn get(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
    fn set(&self, v: f64) {
        self.set_raw(v.clamp(0.0, 1.0));
    }
    fn set_raw(&self, v: f64) {
        self.0.store(v.to_bits(), Ordering::Relaxed);
    }
}

/// Read the 8-byte state both test plugins save (their one parameter).
///
/// # Safety
/// `s` must be null or a live stream.
unsafe fn read_value(s: *mut IBStream) -> Option<f64> {
    // SAFETY: as documented.
    let s = unsafe { ComRef::from_raw(s) }?;
    let mut b = [0u8; 8];
    let mut n = 0;
    // SAFETY: reading into a local 8-byte buffer.
    unsafe { s.read(b.as_mut_ptr() as *mut c_void, 8, &mut n) };
    (n == 8).then(|| f64::from_le_bytes(b))
}

/// # Safety
/// `s` must be null or a live stream.
unsafe fn write_value(s: *mut IBStream, v: f64) -> tresult {
    // SAFETY: as documented.
    let Some(s) = (unsafe { ComRef::from_raw(s) }) else {
        return kInvalidArgument;
    };
    let mut b = v.to_le_bytes();
    // SAFETY: writing from a local 8-byte buffer.
    unsafe { s.write(b.as_mut_ptr() as *mut c_void, 8, std::ptr::null_mut()) }
}

/// The last value of parameter 0 in a block's changes.
///
/// # Safety
/// `changes` must be null or live.
unsafe fn last_change(changes: *mut IParameterChanges) -> Option<f64> {
    // SAFETY: as documented; queues come from the same live object.
    unsafe {
        let changes = ComRef::from_raw(changes)?;
        let mut out = None;
        for i in 0..changes.getParameterCount() {
            let q = ComRef::from_raw(changes.getParameterData(i))?;
            let n = q.getPointCount();
            if q.getParameterId() == 0 && n > 0 {
                let (mut o, mut v) = (0, 0.0);
                if q.getPoint(n - 1, &mut o, &mut v) == kResultOk {
                    out = Some(v);
                }
            }
        }
        out
    }
}

const NO_PROGRAMS: &[(&str, f64)] = &[];
/// Lime Test Tone's factory programs: a name and the level each sets.
const TONE_PROGRAMS: &[(&str, f64)] = &[("Medium", 0.5), ("Quiet", 0.25), ("Loud", 1.0)];

/// The program a normalised program-change value selects.
fn program_index(programs: &[(&str, f64)], value: f64) -> usize {
    let n = programs.len().saturating_sub(1);
    (value.clamp(0.0, 1.0) * n as f64).round() as usize
}

/// Bus description shared by both processors.
///
/// # Safety
/// `bus` must be null or point to a `BusInfo`.
unsafe fn bus_info(
    media: MediaType,
    dir: BusDirection,
    index: i32,
    bus: *mut BusInfo,
    audio_in: bool,
    event_in: bool,
) -> tresult {
    if bus.is_null() || index != 0 {
        return kInvalidArgument;
    }
    // SAFETY: checked non-null.
    let b = unsafe { &mut *bus };
    let audio = media == MediaTypes_::kAudio as i32;
    let input = dir == BusDirections_::kInput as i32;
    let ok = match (audio, input) {
        (true, true) => audio_in,
        (true, false) => true,
        (false, true) => event_in,
        (false, false) => false,
    };
    if !ok {
        return kInvalidArgument;
    }
    b.mediaType = media;
    b.direction = dir;
    b.channelCount = if audio { 2 } else { 16 };
    copy_wstr(if input { "Input" } else { "Output" }, &mut b.name);
    b.busType = BusTypes_::kMain as i32;
    b.flags = BusInfo_::BusFlags_::kDefaultActive as u32;
    kResultOk
}

/// Stereo main channel buffers of bus 0, if present.
///
/// # Safety
/// `buses` must point to `count` live bus buffers of at least `frames` samples.
unsafe fn stereo<'a>(buses: *mut AudioBusBuffers, count: i32, frames: usize) -> Option<[&'a mut [f32]; 2]> {
    if buses.is_null() || count < 1 {
        return None;
    }
    // SAFETY: as documented.
    unsafe {
        let b = &*buses;
        if b.numChannels < 2 {
            return None;
        }
        let ch = std::slice::from_raw_parts(b.__field0.channelBuffers32, 2);
        Some([
            std::slice::from_raw_parts_mut(ch[0], frames),
            std::slice::from_raw_parts_mut(ch[1], frames),
        ])
    }
}

macro_rules! plugin_base {
    ($t:ty) => {
        impl IPluginBaseTrait for $t {
            unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
                kResultOk
            }
            unsafe fn terminate(&self) -> tresult {
                kResultOk
            }
        }
    };
}

/// The audio-processor functions both test plugins implement the same way.
macro_rules! stereo_processor {
    ($t:ty) => {
        impl IAudioProcessorTrait for $t {
            unsafe fn setBusArrangements(
                &self,
                _inputs: *mut SpeakerArrangement,
                _num_ins: i32,
                _outputs: *mut SpeakerArrangement,
                _num_outs: i32,
            ) -> tresult {
                kResultTrue
            }
            unsafe fn getBusArrangement(
                &self,
                _dir: BusDirection,
                index: i32,
                arr: *mut SpeakerArrangement,
            ) -> tresult {
                if index != 0 || arr.is_null() {
                    return kInvalidArgument;
                }
                // SAFETY: checked non-null.
                unsafe { *arr = SpeakerArr::kStereo };
                kResultOk
            }
            unsafe fn canProcessSampleSize(&self, size: i32) -> tresult {
                if size == SymbolicSampleSizes_::kSample32 as i32 {
                    kResultOk
                } else {
                    kNotImplemented
                }
            }
            unsafe fn getLatencySamples(&self) -> u32 {
                0
            }
            unsafe fn setupProcessing(&self, _setup: *mut ProcessSetup) -> tresult {
                kResultOk
            }
            unsafe fn setProcessing(&self, _state: TBool) -> tresult {
                kResultOk
            }
            unsafe fn process(&self, data: *mut ProcessData) -> tresult {
                if data.is_null() {
                    return kInvalidArgument;
                }
                // SAFETY: the host passes valid process data.
                unsafe { self.run(&*data) }
            }
            unsafe fn getTailSamples(&self) -> u32 {
                0
            }
        }
    };
}

/// An edit controller with the normalised parameter `self.value()` (id 0) and, when `$programs`
/// lists two or more `(name, value)` pairs, a program-change parameter (id 1) that sets it.
macro_rules! one_param_controller {
    ($t:ty, $name:expr, $default:expr, $programs:expr) => {
        impl IEditControllerTrait for $t {
            unsafe fn setComponentState(&self, state: *mut IBStream) -> tresult {
                // SAFETY: the host passes a live stream.
                if let Some(v) = unsafe { read_value(state) } {
                    self.value().set(v);
                }
                kResultOk
            }
            unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
                kResultOk
            }
            unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
                kResultOk
            }
            unsafe fn getParameterCount(&self) -> i32 {
                if $programs.len() > 1 { 2 } else { 1 }
            }
            unsafe fn getParameterInfo(&self, index: i32, info: *mut ParameterInfo) -> tresult {
                if index > i32::from($programs.len() > 1) || index < 0 || info.is_null() {
                    return kInvalidArgument;
                }
                // SAFETY: checked non-null.
                let info = unsafe { &mut *info };
                if index == 1 {
                    info.id = 1;
                    copy_wstr("Program", &mut info.title);
                    copy_wstr("Program", &mut info.shortTitle);
                    copy_wstr("", &mut info.units);
                    info.stepCount = $programs.len() as i32 - 1;
                    info.defaultNormalizedValue = 0.0;
                    info.unitId = 0;
                    info.flags =
                        ParameterInfo_::ParameterFlags_::kIsProgramChange | ParameterInfo_::ParameterFlags_::kIsList;
                    return kResultOk;
                }
                info.id = 0;
                copy_wstr($name, &mut info.title);
                copy_wstr($name, &mut info.shortTitle);
                copy_wstr("%", &mut info.units);
                info.stepCount = 0;
                info.defaultNormalizedValue = $default;
                info.unitId = 0;
                info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate;
                kResultOk
            }
            unsafe fn getParamStringByValue(&self, id: u32, value: f64, string: *mut String128) -> tresult {
                if string.is_null() {
                    return kInvalidArgument;
                }
                if id == 1 {
                    let i = program_index($programs, value);
                    // SAFETY: checked non-null.
                    copy_wstr($programs.get(i).map_or("", |p| p.0), unsafe { &mut *string });
                    return kResultOk;
                }
                if id != 0 {
                    return kInvalidArgument;
                }
                // SAFETY: checked non-null.
                copy_wstr(&format!("{:.0}%", value * 100.0), unsafe { &mut *string });
                kResultOk
            }
            unsafe fn getParamValueByString(&self, _id: u32, _string: *mut TChar, _value: *mut f64) -> tresult {
                kNotImplemented
            }
            unsafe fn normalizedParamToPlain(&self, _id: u32, value: f64) -> f64 {
                value
            }
            unsafe fn plainParamToNormalized(&self, _id: u32, value: f64) -> f64 {
                value
            }
            unsafe fn getParamNormalized(&self, id: u32) -> f64 {
                match id {
                    0 => self.value().get(),
                    1 => {
                        let v = self.value().get();
                        let n = $programs.len().saturating_sub(1).max(1) as f64;
                        $programs
                            .iter()
                            .position(|p| (p.1 - v).abs() < 1e-9)
                            .map_or(0.0, |i| i as f64 / n)
                    }
                    _ => 0.0,
                }
            }
            unsafe fn setParamNormalized(&self, id: u32, value: f64) -> tresult {
                match id {
                    0 => self.value().set(value),
                    1 if $programs.len() > 1 => {
                        if let Some(p) = $programs.get(program_index($programs, value)) {
                            self.value().set(p.1);
                        }
                    }
                    _ => return kInvalidArgument,
                }
                kResultOk
            }
            unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
                self.set_handler(handler);
                kResultOk
            }
            unsafe fn createView(&self, _name: *const c_char) -> *mut IPlugView {
                self.create_view()
            }
        }
    };
}

macro_rules! component {
    ($t:ty, $controller:expr, $audio_in:expr, $event_in:expr) => {
        impl IComponentTrait for $t {
            unsafe fn getControllerClassId(&self, class_id: *mut TUID) -> tresult {
                match ($controller, class_id.is_null()) {
                    (Some(cid), false) => {
                        // SAFETY: checked non-null.
                        unsafe { *class_id = cid };
                        kResultOk
                    }
                    _ => kResultFalse,
                }
            }
            unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
                kResultOk
            }
            unsafe fn getBusCount(&self, media: MediaType, dir: BusDirection) -> i32 {
                let audio = media == MediaTypes_::kAudio as i32;
                let input = dir == BusDirections_::kInput as i32;
                match (audio, input) {
                    (true, true) => $audio_in as i32,
                    (true, false) => 1,
                    (false, true) => $event_in as i32,
                    (false, false) => 0,
                }
            }
            unsafe fn getBusInfo(&self, media: MediaType, dir: BusDirection, index: i32, bus: *mut BusInfo) -> tresult {
                // SAFETY: the host passes a `BusInfo` or null.
                unsafe { bus_info(media, dir, index, bus, $audio_in, $event_in) }
            }
            unsafe fn getRoutingInfo(&self, _i: *mut RoutingInfo, _o: *mut RoutingInfo) -> tresult {
                kNotImplemented
            }
            unsafe fn activateBus(&self, _m: MediaType, _d: BusDirection, _i: i32, _s: TBool) -> tresult {
                kResultOk
            }
            unsafe fn setActive(&self, _state: TBool) -> tresult {
                kResultOk
            }
            unsafe fn setState(&self, state: *mut IBStream) -> tresult {
                // SAFETY: the host passes a live stream.
                match unsafe { read_value(state) } {
                    Some(v) => {
                        self.value().set(v);
                        kResultOk
                    }
                    None => kResultFalse,
                }
            }
            unsafe fn getState(&self, state: *mut IBStream) -> tresult {
                // SAFETY: the host passes a live stream.
                unsafe { write_value(state, self.value().get()) }
            }
        }
    };
}

struct Gain {
    gain: Value,
}

impl Class for Gain {
    type Interfaces = (IComponent, IAudioProcessor);
}

impl Gain {
    fn value(&self) -> &Value {
        &self.gain
    }

    /// # Safety
    /// `d` must describe live buffers.
    unsafe fn run(&self, d: &ProcessData) -> tresult {
        // SAFETY: as documented.
        unsafe {
            if let Some(v) = last_change(d.inputParameterChanges) {
                self.gain.set(v);
            }
            let n = d.numSamples.max(0) as usize;
            let g = self.gain.get() as f32;
            if let (Some([il, ir]), Some([ol, or])) =
                (stereo(d.inputs, d.numInputs, n), stereo(d.outputs, d.numOutputs, n))
            {
                for i in 0..n {
                    ol[i] = il[i] * g;
                    or[i] = ir[i] * g;
                }
            }
        }
        kResultOk
    }
}

plugin_base!(Gain);
component!(Gain, Some(GAIN_CTRL_CID), true, false);
stereo_processor!(Gain);

struct GainController {
    gain: Arc<Value>,
    /// The host's handler (owned by the host, which clears it before it goes).
    handler: Cell<*mut IComponentHandler>,
}

impl Class for GainController {
    type Interfaces = (IEditController,);
}

impl GainController {
    fn value(&self) -> &Value {
        &self.gain
    }

    fn set_handler(&self, handler: *mut IComponentHandler) {
        self.handler.set(handler);
    }

    fn create_view(&self) -> *mut IPlugView {
        ComWrapper::new(TestView {
            gain: self.gain.clone(),
            handler: self.handler.get(),
            frame: Cell::new(std::ptr::null_mut()),
            size: Cell::new((EDITOR_SIZE.0, EDITOR_SIZE.1)),
            attached: Cell::new(false),
        })
        .to_com_ptr::<IPlugView>()
        .map(|p| p.into_raw())
        .unwrap_or(std::ptr::null_mut())
    }
}

/// The size Lime Test Gain's editor opens at.
pub const EDITOR_SIZE: (i32, i32) = (320, 200);
/// The size it asks for when it gets the key `r`.
pub const EDITOR_RESIZED: (i32, i32) = (400, 260);

/// Lime Test Gain's editor. It draws nothing (and never touches its parent, so it can be attached
/// without a window); keys stand in for the mouse: `-` sets the gain to 0.5 as a user's edit
/// would, `r` asks the host for [`EDITOR_RESIZED`].
struct TestView {
    gain: Arc<Value>,
    handler: *mut IComponentHandler,
    frame: Cell<*mut IPlugFrame>,
    size: Cell<(i32, i32)>,
    attached: Cell<bool>,
}

impl Class for TestView {
    type Interfaces = (IPlugView,);
}

impl IPlugViewTrait for TestView {
    unsafe fn isPlatformTypeSupported(&self, _type: FIDString) -> tresult {
        kResultTrue
    }
    unsafe fn attached(&self, _parent: *mut c_void, _type: FIDString) -> tresult {
        if self.attached.replace(true) {
            return kResultFalse;
        }
        kResultOk
    }
    unsafe fn removed(&self) -> tresult {
        self.attached.set(false);
        kResultOk
    }
    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }
    unsafe fn onKeyDown(&self, key: char16, _code: int16, _mods: int16) -> tresult {
        // SAFETY: the host's handler and frame outlive the view (the host's contract).
        unsafe {
            match char::from_u32(u32::from(key)) {
                Some('-') => {
                    self.gain.set(0.5);
                    if let Some(h) = ComRef::from_raw(self.handler) {
                        h.beginEdit(0);
                        h.performEdit(0, 0.5);
                        h.endEdit(0);
                    }
                    kResultTrue
                }
                Some('r') => {
                    let mut r = ViewRect {
                        left: 0,
                        top: 0,
                        right: EDITOR_RESIZED.0,
                        bottom: EDITOR_RESIZED.1,
                    };
                    if let Some(f) = ComRef::from_raw(self.frame.get()) {
                        f.resizeView(std::ptr::null_mut(), &mut r);
                    }
                    kResultTrue
                }
                _ => kResultFalse,
            }
        }
    }
    unsafe fn onKeyUp(&self, _key: char16, _code: int16, _mods: int16) -> tresult {
        kResultFalse
    }
    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }
        let (w, h) = self.size.get();
        // SAFETY: checked non-null.
        unsafe {
            *size = ViewRect {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            }
        };
        kResultOk
    }
    unsafe fn onSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null.
        let r = unsafe { &*size };
        self.size.set((r.right - r.left, r.bottom - r.top));
        kResultOk
    }
    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }
    unsafe fn setFrame(&self, frame: *mut IPlugFrame) -> tresult {
        self.frame.set(frame);
        kResultOk
    }
    unsafe fn canResize(&self) -> tresult {
        kResultTrue
    }
    unsafe fn checkSizeConstraint(&self, _rect: *mut ViewRect) -> tresult {
        kResultOk
    }
}

plugin_base!(GainController);
one_param_controller!(GainController, "Gain", 1.0, NO_PROGRAMS);

#[derive(Clone, Copy)]
struct Voice {
    key: Option<i16>,
    velocity: f32,
    phase: f64,
}

struct Tone {
    level: Value,
    sample_rate: Value,
    voice: Mutex<Voice>,
}

impl Class for Tone {
    type Interfaces = (IComponent, IAudioProcessor, IEditController);
}

impl Tone {
    fn value(&self) -> &Value {
        &self.level
    }

    fn set_handler(&self, _handler: *mut IComponentHandler) {}

    /// Lime Test Tone has no editor.
    fn create_view(&self) -> *mut IPlugView {
        std::ptr::null_mut()
    }

    /// # Safety
    /// `d` must describe live buffers and events.
    unsafe fn run(&self, d: &ProcessData) -> tresult {
        // SAFETY: as documented.
        unsafe {
            if let Some(v) = last_change(d.inputParameterChanges) {
                self.level.set(v);
            }
            let n = d.numSamples.max(0) as usize;
            let Some([ol, or]) = stereo(d.outputs, d.numOutputs, n) else {
                return kResultOk;
            };
            // A test plugin may lock: it is not host code, and nothing else holds the lock.
            let Ok(mut voice) = self.voice.try_lock() else {
                return kResultOk;
            };
            let events = ComRef::from_raw(d.inputEvents);
            let count = events.map(|e| e.getEventCount()).unwrap_or(0);
            let mut next = 0;
            let sr = self.sample_rate.get();
            let level = self.level.get() as f32;
            for i in 0..n {
                while next < count {
                    let mut e: Event = std::mem::zeroed();
                    if let Some(list) = events
                        && list.getEvent(next, &mut e) == kResultOk
                    {
                        if e.sampleOffset as usize > i {
                            break;
                        }
                        if e.r#type == Event_::EventTypes_::kNoteOnEvent as u16 {
                            voice.key = Some(e.__field0.noteOn.pitch);
                            voice.velocity = e.__field0.noteOn.velocity;
                            voice.phase = 0.0;
                        } else if e.r#type == Event_::EventTypes_::kNoteOffEvent as u16
                            && voice.key == Some(e.__field0.noteOff.pitch)
                        {
                            voice.key = None;
                        }
                    }
                    next += 1;
                }
                let s = match voice.key {
                    Some(k) => {
                        let f = 440.0 * 2f64.powf((k as f64 - 69.0) / 12.0);
                        voice.phase = (voice.phase + f / sr).fract();
                        (voice.phase * std::f64::consts::TAU).sin() as f32 * level * voice.velocity
                    }
                    None => 0.0,
                };
                ol[i] = s;
                or[i] = s;
            }
        }
        kResultOk
    }
}

plugin_base!(Tone);
component!(Tone, None::<TUID>, false, true);
one_param_controller!(Tone, "Level", 0.5, TONE_PROGRAMS);

impl IAudioProcessorTrait for Tone {
    unsafe fn setBusArrangements(
        &self,
        _inputs: *mut SpeakerArrangement,
        _num_ins: i32,
        _outputs: *mut SpeakerArrangement,
        _num_outs: i32,
    ) -> tresult {
        kResultTrue
    }
    unsafe fn getBusArrangement(&self, _dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> tresult {
        if index != 0 || arr.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null.
        unsafe { *arr = SpeakerArr::kStereo };
        kResultOk
    }
    unsafe fn canProcessSampleSize(&self, size: i32) -> tresult {
        if size == SymbolicSampleSizes_::kSample32 as i32 {
            kResultOk
        } else {
            kNotImplemented
        }
    }
    unsafe fn getLatencySamples(&self) -> u32 {
        0
    }
    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        if !setup.is_null() {
            // SAFETY: checked non-null.
            self.sample_rate.set_raw(unsafe { (*setup).sampleRate });
        }
        kResultOk
    }
    unsafe fn setProcessing(&self, _state: TBool) -> tresult {
        if let Ok(mut v) = self.voice.lock() {
            v.key = None;
        }
        kResultOk
    }
    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        if data.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: the host passes valid process data.
        unsafe { self.run(&*data) }
    }
    unsafe fn getTailSamples(&self) -> u32 {
        0
    }
}

struct Factory;

impl Class for Factory {
    type Interfaces = (IPluginFactory2,);
}

const CLASSES: [(TUID, &str, &str, &str); 3] = [
    (GAIN_CID, "Audio Module Class", "Lime Test Gain", "Fx"),
    (GAIN_CTRL_CID, "Component Controller Class", "Lime Test Gain", ""),
    (TONE_CID, "Audio Module Class", "Lime Test Tone", "Instrument|Synth"),
];

impl IPluginFactoryTrait for Factory {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null.
        let info = unsafe { &mut *info };
        copy_cstr("LimeShot", &mut info.vendor);
        copy_cstr("https://github.com/fruitapps/limeshot", &mut info.url);
        copy_cstr("", &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as i32;
        kResultOk
    }

    unsafe fn countClasses(&self) -> i32 {
        CLASSES.len() as i32
    }

    unsafe fn getClassInfo(&self, index: i32, info: *mut PClassInfo) -> tresult {
        let Some((cid, cat, name, _)) = CLASSES.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        if info.is_null() || index < 0 {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null.
        let info = unsafe { &mut *info };
        info.cid = *cid;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as i32;
        copy_cstr(cat, &mut info.category);
        copy_cstr(name, &mut info.name);
        kResultOk
    }

    unsafe fn createInstance(&self, cid: FIDString, iid: FIDString, obj: *mut *mut c_void) -> tresult {
        if cid.is_null() || iid.is_null() || obj.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: class ids are 16 bytes.
        let cid = unsafe { *(cid as *const TUID) };
        let unknown = if cid == GAIN_CID {
            ComWrapper::new(Gain { gain: Value::new(1.0) }).to_com_ptr::<FUnknown>()
        } else if cid == GAIN_CTRL_CID {
            ComWrapper::new(GainController {
                gain: Arc::new(Value::new(1.0)),
                handler: Cell::new(std::ptr::null_mut()),
            })
            .to_com_ptr::<FUnknown>()
        } else if cid == TONE_CID {
            ComWrapper::new(Tone {
                level: Value::new(0.5),
                sample_rate: Value::new(48_000.0),
                voice: Mutex::new(Voice {
                    key: None,
                    velocity: 0.0,
                    phase: 0.0,
                }),
            })
            .to_com_ptr::<FUnknown>()
        } else {
            None
        };
        let Some(unknown) = unknown else {
            // SAFETY: checked non-null.
            unsafe { *obj = std::ptr::null_mut() };
            return kNoInterface;
        };
        let p = unknown.as_ptr();
        // SAFETY: a live object; `queryInterface` adds the reference the caller owns.
        unsafe { ((*(*p).vtbl).queryInterface)(p, iid as *const TUID, obj) }
    }
}

impl IPluginFactory2Trait for Factory {
    unsafe fn getClassInfo2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        let Some((cid, cat, name, sub)) = CLASSES.get(index.max(0) as usize) else {
            return kInvalidArgument;
        };
        if info.is_null() || index < 0 {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null.
        let info = unsafe { &mut *info };
        info.cid = *cid;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as i32;
        copy_cstr(cat, &mut info.category);
        copy_cstr(name, &mut info.name);
        info.classFlags = 0;
        copy_cstr(sub, &mut info.subCategories);
        copy_cstr("LimeShot", &mut info.vendor);
        copy_cstr(env!("CARGO_PKG_VERSION"), &mut info.version);
        copy_cstr("VST 3.8.0", &mut info.sdkVersion);
        kResultOk
    }
}
