//! COM objects the host gives to plugins: the host context, messages, memory streams, the
//! component handler, and the fixed-capacity parameter-change and event lists used in `process`.

use std::cell::{Cell, UnsafeCell};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, c_void};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};

use vst3::Steinberg::Vst::IAttributeList_::AttrID;
use vst3::Steinberg::Vst::*;
use vst3::Steinberg::*;
use vst3::{Class, ComWrapper};

use crate::util::{copy_wstr, iid};

/// Answer `queryInterface`-style object creation: write `obj` as interface `iid` of `wrapper`.
///
/// # Safety
/// `want` must point to a 16-byte id and `obj` to writable storage.
unsafe fn hand_out<C: Class>(wrapper: ComWrapper<C>, want: *const TUID, obj: *mut *mut c_void) -> tresult {
    let Some(unknown) = wrapper.to_com_ptr::<FUnknown>() else {
        return kNoInterface;
    };
    let ptr = unknown.as_ptr();
    // SAFETY: `ptr` is a live object; `queryInterface` adds the reference the caller owns.
    unsafe { ((*(*ptr).vtbl).queryInterface)(ptr, want, obj) }
}

/// The host context passed to `initialize`.
pub(crate) struct HostApp;

impl Class for HostApp {
    type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for HostApp {
    unsafe fn getName(&self, name: *mut String128) -> tresult {
        if name.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null; the caller owns a `String128`.
        copy_wstr("LimeShot", unsafe { &mut *name });
        kResultOk
    }

    unsafe fn createInstance(&self, cid: *mut TUID, want: *mut TUID, obj: *mut *mut c_void) -> tresult {
        if cid.is_null() || want.is_null() || obj.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null 16-byte ids.
        let cid = unsafe { *cid };
        if cid == iid::<IMessage>() {
            // SAFETY: arguments checked above.
            unsafe { hand_out(ComWrapper::new(Message::default()), want, obj) }
        } else if cid == iid::<IAttributeList>() {
            // SAFETY: arguments checked above.
            unsafe { hand_out(ComWrapper::new(AttributeList::default()), want, obj) }
        } else {
            // SAFETY: `obj` checked non-null.
            unsafe { *obj = std::ptr::null_mut() };
            kResultFalse
        }
    }
}

#[derive(Clone)]
enum Attr {
    Int(i64),
    Float(f64),
    String(Vec<TChar>),
    Binary(Vec<u8>),
}

/// A message attribute list (used by plugins whose processor and controller talk through the
/// host's `IConnectionPoint` connection).
#[derive(Default)]
pub(crate) struct AttributeList {
    attrs: Mutex<HashMap<Vec<u8>, Attr>>,
}

impl Class for AttributeList {
    type Interfaces = (IAttributeList,);
}

/// # Safety
/// `id` must be null or a NUL-terminated string.
unsafe fn key(id: AttrID) -> Option<Vec<u8>> {
    // SAFETY: as documented.
    (!id.is_null()).then(|| unsafe { CStr::from_ptr(id) }.to_bytes().to_vec())
}

impl AttributeList {
    fn set(&self, id: AttrID, v: Attr) -> tresult {
        // SAFETY: plugins pass NUL-terminated ids.
        let Some(k) = (unsafe { key(id) }) else {
            return kInvalidArgument;
        };
        match self.attrs.lock() {
            Ok(mut a) => {
                a.insert(k, v);
                kResultOk
            }
            Err(_) => kInternalError,
        }
    }

    fn get(&self, id: AttrID) -> Option<Attr> {
        // SAFETY: plugins pass NUL-terminated ids.
        let k = unsafe { key(id) }?;
        self.attrs.lock().ok()?.get(&k).cloned()
    }
}

impl IAttributeListTrait for AttributeList {
    unsafe fn setInt(&self, id: AttrID, value: i64) -> tresult {
        self.set(id, Attr::Int(value))
    }

    unsafe fn getInt(&self, id: AttrID, value: *mut i64) -> tresult {
        match self.get(id) {
            Some(Attr::Int(v)) if !value.is_null() => {
                // SAFETY: checked non-null.
                unsafe { *value = v };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setFloat(&self, id: AttrID, value: f64) -> tresult {
        self.set(id, Attr::Float(value))
    }

    unsafe fn getFloat(&self, id: AttrID, value: *mut f64) -> tresult {
        match self.get(id) {
            Some(Attr::Float(v)) if !value.is_null() => {
                // SAFETY: checked non-null.
                unsafe { *value = v };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setString(&self, id: AttrID, string: *const TChar) -> tresult {
        if string.is_null() {
            return kInvalidArgument;
        }
        let mut s = Vec::new();
        // SAFETY: a NUL-terminated UTF-16 string from the plugin.
        unsafe {
            let mut p = string;
            while *p != 0 {
                s.push(*p);
                p = p.add(1);
            }
        }
        self.set(id, Attr::String(s))
    }

    unsafe fn getString(&self, id: AttrID, string: *mut TChar, size_in_bytes: u32) -> tresult {
        let cap = size_in_bytes as usize / std::mem::size_of::<TChar>();
        match self.get(id) {
            Some(Attr::String(s)) if !string.is_null() && cap > 0 => {
                let n = s.len().min(cap - 1);
                // SAFETY: the caller's buffer holds `cap` characters.
                unsafe {
                    std::ptr::copy_nonoverlapping(s.as_ptr(), string, n);
                    *string.add(n) = 0;
                }
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setBinary(&self, id: AttrID, data: *const c_void, size_in_bytes: u32) -> tresult {
        let bytes = if data.is_null() || size_in_bytes == 0 {
            Vec::new()
        } else {
            // SAFETY: the plugin passes `size_in_bytes` readable bytes.
            unsafe { std::slice::from_raw_parts(data as *const u8, size_in_bytes as usize) }.to_vec()
        };
        self.set(id, Attr::Binary(bytes))
    }

    unsafe fn getBinary(&self, id: AttrID, data: *mut *const c_void, size_in_bytes: *mut u32) -> tresult {
        if data.is_null() || size_in_bytes.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: plugins pass NUL-terminated ids.
        let Some(k) = (unsafe { key(id) }) else {
            return kInvalidArgument;
        };
        let Ok(attrs) = self.attrs.lock() else {
            return kInternalError;
        };
        match attrs.get(&k) {
            Some(Attr::Binary(b)) => {
                // The pointer stays valid while the attribute is not replaced, as the API asks.
                // SAFETY: both out-pointers checked non-null.
                unsafe {
                    *data = b.as_ptr() as *const c_void;
                    *size_in_bytes = b.len() as u32;
                }
                kResultOk
            }
            _ => kResultFalse,
        }
    }
}

/// A message between a plugin's processor and controller.
pub(crate) struct Message {
    id: Mutex<Vec<u8>>,
    attributes: ComWrapper<AttributeList>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: Mutex::new(vec![0]),
            attributes: ComWrapper::new(AttributeList::default()),
        }
    }
}

impl Class for Message {
    type Interfaces = (IMessage,);
}

impl IMessageTrait for Message {
    unsafe fn getMessageID(&self) -> FIDString {
        // The buffer is only replaced by `setMessageID`; plugins read the id right away.
        self.id
            .lock()
            .map(|id| id.as_ptr() as FIDString)
            .unwrap_or(std::ptr::null())
    }

    unsafe fn setMessageID(&self, id: FIDString) {
        if id.is_null() {
            return;
        }
        // SAFETY: a NUL-terminated id from the plugin.
        let bytes = unsafe { CStr::from_ptr(id) }.to_bytes_with_nul().to_vec();
        if let Ok(mut m) = self.id.lock() {
            *m = bytes;
        }
    }

    unsafe fn getAttributes(&self) -> *mut IAttributeList {
        // Not add-ref'd: the message owns its attribute list, as in the SDK's host classes.
        self.attributes
            .as_com_ref::<IAttributeList>()
            .map(|r| r.as_ptr())
            .unwrap_or(std::ptr::null_mut())
    }
}

/// A growable in-memory `IBStream` for plugin state.
#[derive(Default)]
pub(crate) struct MemoryStream {
    data: Mutex<(Vec<u8>, usize)>,
}

impl MemoryStream {
    pub(crate) fn with_data(bytes: Vec<u8>) -> Self {
        Self {
            data: Mutex::new((bytes, 0)),
        }
    }

    pub(crate) fn take(&self) -> Vec<u8> {
        self.data
            .lock()
            .map(|mut d| std::mem::take(&mut d.0))
            .unwrap_or_default()
    }
}

impl Class for MemoryStream {
    type Interfaces = (IBStream,);
}

impl IBStreamTrait for MemoryStream {
    unsafe fn read(&self, buffer: *mut c_void, num_bytes: i32, num_read: *mut i32) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kInternalError;
        };
        let (data, pos) = &mut *d;
        let n = (num_bytes.max(0) as usize).min(data.len().saturating_sub(*pos));
        if n > 0 && !buffer.is_null() {
            // SAFETY: the caller's buffer holds `num_bytes` bytes.
            unsafe { std::ptr::copy_nonoverlapping(data.as_ptr().add(*pos), buffer as *mut u8, n) };
        }
        *pos += n;
        if !num_read.is_null() {
            // SAFETY: checked non-null.
            unsafe { *num_read = n as i32 };
        }
        kResultOk
    }

    unsafe fn write(&self, buffer: *mut c_void, num_bytes: i32, num_written: *mut i32) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kInternalError;
        };
        let (data, pos) = &mut *d;
        let n = num_bytes.max(0) as usize;
        if buffer.is_null() && n > 0 {
            return kInvalidArgument;
        }
        if data.len() < *pos + n {
            data.resize(*pos + n, 0);
        }
        if n > 0 {
            // SAFETY: the plugin passes `num_bytes` readable bytes.
            unsafe { std::ptr::copy_nonoverlapping(buffer as *const u8, data.as_mut_ptr().add(*pos), n) };
        }
        *pos += n;
        if !num_written.is_null() {
            // SAFETY: checked non-null.
            unsafe { *num_written = n as i32 };
        }
        kResultOk
    }

    unsafe fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> tresult {
        let Ok(mut d) = self.data.lock() else {
            return kInternalError;
        };
        use IBStream_::IStreamSeekMode_::{kIBSeekCur, kIBSeekEnd, kIBSeekSet};
        // The enum's integer type differs between platforms, so compare as i64.
        let base = match mode as i64 {
            m if m == kIBSeekSet as i64 => 0,
            m if m == kIBSeekCur as i64 => d.1 as i64,
            m if m == kIBSeekEnd as i64 => d.0.len() as i64,
            _ => return kInvalidArgument,
        };
        let new = base.saturating_add(pos);
        if new < 0 {
            return kInvalidArgument;
        }
        d.1 = new as usize;
        if !result.is_null() {
            // SAFETY: checked non-null.
            unsafe { *result = new };
        }
        kResultOk
    }

    unsafe fn tell(&self, pos: *mut i64) -> tresult {
        if pos.is_null() {
            return kInvalidArgument;
        }
        let Ok(d) = self.data.lock() else {
            return kInternalError;
        };
        // SAFETY: checked non-null.
        unsafe { *pos = d.1 as i64 };
        kResultOk
    }
}

/// Receives parameter edits made in a plugin's own editor.
#[derive(Default)]
pub(crate) struct ComponentHandler {
    pub(crate) edits: Mutex<Vec<(u32, f64)>>,
    pub(crate) active_edits: Mutex<HashSet<u32>>,
    pub(crate) restart_flags: AtomicI32,
}

impl Class for ComponentHandler {
    type Interfaces = (IComponentHandler,);
}

impl IComponentHandlerTrait for ComponentHandler {
    unsafe fn beginEdit(&self, id: ParamID) -> tresult {
        if let Ok(mut active) = self.active_edits.lock() {
            active.insert(id);
        }
        kResultOk
    }

    unsafe fn performEdit(&self, id: ParamID, value: ParamValue) -> tresult {
        if let Ok(mut e) = self.edits.lock() {
            e.push((id, value));
        }
        kResultOk
    }

    unsafe fn endEdit(&self, id: ParamID) -> tresult {
        if let Ok(mut active) = self.active_edits.lock() {
            active.remove(&id);
        }
        kResultOk
    }

    unsafe fn restartComponent(&self, flags: i32) -> tresult {
        self.restart_flags.fetch_or(flags, Ordering::Relaxed);
        kResultOk
    }
}

/// Points per parameter per block; later points for the same parameter replace the last one.
const QUEUE_POINTS: usize = 16;

/// One parameter's changes in a block. Only touched on the audio thread during `process`.
pub(crate) struct ParamQueue {
    id: Cell<u32>,
    len: Cell<usize>,
    points: UnsafeCell<[(i32, f64); QUEUE_POINTS]>,
}

impl Class for ParamQueue {
    type Interfaces = (IParamValueQueue,);
}

impl ParamQueue {
    fn new() -> Self {
        Self {
            id: Cell::new(0),
            len: Cell::new(0),
            points: UnsafeCell::new([(0, 0.0); QUEUE_POINTS]),
        }
    }

    fn push(&self, offset: i32, value: f64) -> i32 {
        // SAFETY: single-threaded use (see the type docs); no reference into `points` escapes.
        let points = unsafe { &mut *self.points.get() };
        let n = self.len.get();
        // Points must be in order; a point at or before the last one replaces it.
        if n > 0 && points[n - 1].0 >= offset {
            points[n - 1] = (points[n - 1].0, value);
            return n as i32 - 1;
        }
        if n == QUEUE_POINTS {
            points[n - 1] = (offset, value);
            return n as i32 - 1;
        }
        points[n] = (offset, value);
        self.len.set(n + 1);
        n as i32
    }
}

impl IParamValueQueueTrait for ParamQueue {
    unsafe fn getParameterId(&self) -> ParamID {
        self.id.get()
    }

    unsafe fn getPointCount(&self) -> i32 {
        self.len.get() as i32
    }

    unsafe fn getPoint(&self, index: i32, offset: *mut i32, value: *mut ParamValue) -> tresult {
        if index < 0 || index as usize >= self.len.get() || offset.is_null() || value.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: index in range; out-pointers checked non-null.
        unsafe {
            let (o, v) = (*self.points.get())[index as usize];
            *offset = o;
            *value = v;
        }
        kResultOk
    }

    unsafe fn addPoint(&self, offset: i32, value: ParamValue, index: *mut i32) -> tresult {
        let i = self.push(offset, value);
        if !index.is_null() {
            // SAFETY: checked non-null.
            unsafe { *index = i };
        }
        kResultOk
    }
}

/// The parameter changes of one block, with a fixed number of queues (preallocated).
pub(crate) struct ParamChanges {
    queues: Vec<ComWrapper<ParamQueue>>,
    used: Cell<usize>,
}

impl Class for ParamChanges {
    type Interfaces = (IParameterChanges,);
}

impl ParamChanges {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            queues: (0..capacity).map(|_| ComWrapper::new(ParamQueue::new())).collect(),
            used: Cell::new(0),
        }
    }

    pub(crate) fn clear(&self) {
        for q in &self.queues[..self.used.get()] {
            q.len.set(0);
        }
        self.used.set(0);
    }

    /// Add a point for `id`. Returns false when every queue is in use.
    pub(crate) fn add(&self, id: u32, offset: i32, value: f64) -> bool {
        match self.queue_index(id) {
            Some(i) => {
                self.queues[i].push(offset, value);
                true
            }
            None => false,
        }
    }

    fn queue_index(&self, id: u32) -> Option<usize> {
        let used = self.used.get();
        if let Some(i) = self.queues[..used].iter().position(|q| q.id.get() == id) {
            return Some(i);
        }
        if used == self.queues.len() {
            return None;
        }
        self.queues[used].id.set(id);
        self.queues[used].len.set(0);
        self.used.set(used + 1);
        Some(used)
    }

    fn queue_ptr(&self, i: usize) -> *mut IParamValueQueue {
        self.queues
            .get(i)
            .and_then(|q| q.as_com_ref::<IParamValueQueue>())
            .map(|r| r.as_ptr())
            .unwrap_or(std::ptr::null_mut())
    }
}

impl IParameterChangesTrait for ParamChanges {
    unsafe fn getParameterCount(&self) -> i32 {
        self.used.get() as i32
    }

    unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
        if index < 0 || index as usize >= self.used.get() {
            return std::ptr::null_mut();
        }
        self.queue_ptr(index as usize)
    }

    unsafe fn addParameterData(&self, id: *const ParamID, index: *mut i32) -> *mut IParamValueQueue {
        if id.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: checked non-null.
        let Some(i) = self.queue_index(unsafe { *id }) else {
            return std::ptr::null_mut();
        };
        if !index.is_null() {
            // SAFETY: checked non-null.
            unsafe { *index = i as i32 };
        }
        self.queue_ptr(i)
    }
}

/// A block's note events, preallocated.
pub(crate) struct EventList {
    events: UnsafeCell<Vec<Event>>,
    telemetry: UnsafeCell<limeshot_dsp::PluginTelemetry>,
}

impl Class for EventList {
    type Interfaces = (IEventList,);
}

impl EventList {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            events: UnsafeCell::new(Vec::with_capacity(capacity)),
            telemetry: UnsafeCell::new(limeshot_dsp::PluginTelemetry::default()),
        }
    }

    /// Copy completed block statistics without retaining borrowed event pointers.
    pub(crate) fn telemetry(&self) -> limeshot_dsp::PluginTelemetry {
        // SAFETY: only the processor thread reads/writes this event list during processing.
        unsafe { *self.telemetry.get() }
    }
    pub(crate) fn clear(&self) {
        // SAFETY: only the processor thread reads/writes this event list during processing.
        unsafe {
            *self.telemetry.get() = limeshot_dsp::PluginTelemetry::default();
        }
        // SAFETY: single-threaded use during `process`; no reference escapes.
        unsafe { (*self.events.get()).clear() };
    }

    /// Append `e` unless the list is full (never reallocates).
    pub(crate) fn push(&self, e: Event) -> bool {
        // SAFETY: as in `clear`.
        let events = unsafe { &mut *self.events.get() };
        if events.len() < events.capacity() {
            events.push(e);
            true
        } else {
            false
        }
    }
}

impl IEventListTrait for EventList {
    unsafe fn getEventCount(&self) -> i32 {
        // SAFETY: as in `clear`.
        unsafe { (&*self.events.get()).len() as i32 }
    }

    unsafe fn getEvent(&self, index: i32, e: *mut Event) -> tresult {
        // SAFETY: as in `clear`; `e` checked non-null.
        unsafe {
            match (&*self.events.get()).get(index.max(0) as usize) {
                Some(ev) if index >= 0 && !e.is_null() => {
                    *e = *ev;
                    kResultOk
                }
                _ => kInvalidArgument,
            }
        }
    }

    unsafe fn addEvent(&self, e: *mut Event) -> tresult {
        if e.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null; the plugin owns a live event for this call.
        let event = unsafe { *e };
        if event.r#type == Event_::EventTypes_::kDataEvent as u16 {
            // SAFETY: data is the active union member; bytes remain live during addEvent.
            let data = unsafe { event.__field0.data };
            if data.r#type == 0 && data.size == 22 && !data.bytes.is_null() {
                // SAFETY: the VST3 caller supplies `size` readable bytes for this call.
                let bytes = unsafe { std::slice::from_raw_parts(data.bytes, 22) };
                if let Some(value) = limeshot_dsp::PluginTelemetry::decode(bytes) {
                    // SAFETY: event lists are accessed only on the processing thread.
                    unsafe {
                        *self.telemetry.get() = value;
                    }
                }
            }
            // Data pointers are borrowed. Never retain them after this callback.
            return kResultOk;
        }
        if self.push(event) { kResultOk } else { kOutOfMemory }
    }
}

#[cfg(test)]
mod gesture_tests {
    use super::*;

    #[test]
    fn overlapping_native_gestures_finish_only_after_all_controls_end() {
        let handler = ComponentHandler::default();
        // SAFETY: these callbacks receive only plain IDs/values and operate on this live handler.
        unsafe {
            handler.beginEdit(7);
            handler.beginEdit(7); // A duplicate Begin cannot leave capture permanently stuck.
            handler.beginEdit(8);
            handler.performEdit(7, 0.5);
            handler.endEdit(7);
        }
        assert_eq!(handler.active_edits.lock().unwrap().len(), 1);
        assert_eq!(*handler.edits.lock().unwrap(), vec![(7, 0.5)]);
        // SAFETY: same live handler and plain parameter ID.
        unsafe {
            handler.endEdit(8);
            handler.endEdit(8);
        }
        assert!(handler.active_edits.lock().unwrap().is_empty());
    }
}
