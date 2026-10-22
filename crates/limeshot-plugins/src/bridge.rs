//! Plugins in a separate process (FL Studio's "bridged" plugins): a plugin that crashes or hangs
//! takes only its own process down, and LimeShot keeps running with that slot silent.
//!
//! The parent starts a child (`<exe> --plugin-bridge <socket>`), which connects twice to a Unix
//! socket the parent listens on: a control connection (the main thread's calls: parameters,
//! state, presets) and an audio connection (one request and one reply per block). The child loads
//! the plugin on its main thread and processes on a second thread, as LimeShot does in-process.
//! A reply that does not come within [`AUDIO_TIMEOUT`], or a broken connection, marks the plugin
//! crashed; its processor then outputs silence and the instance answers with defaults.
//!
//! Messages are length-prefixed byte strings in a small fixed encoding (little-endian). The audio
//! path reuses preallocated buffers, so the parent's audio thread does not allocate; it does make
//! socket calls, as every plugin bridge does.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{NoteEvent, NoteEventKind, ParamInfo, PluginError, Transport, Vst3Instance, Vst3Module, VstPreset};

/// The command-line flag that makes LimeShot run as a plugin bridge.
pub const BRIDGE_FLAG: &str = "--plugin-bridge";
/// How long a block may take in the child before the plugin counts as hung.
pub const AUDIO_TIMEOUT: Duration = Duration::from_millis(500);
/// How long a control call may take (loading a large state, say).
const CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
/// How long the child may take to start and load the plugin.
const START_TIMEOUT: Duration = Duration::from_secs(30);
/// The largest message accepted (a plugin state can be big).
const MAX_MESSAGE: usize = 256 << 20;
const MAX_EVENTS: usize = 512;
const MAX_PARAMS: usize = 1024;
/// Parameter values are refreshed from the child at most this often.
const REFRESH: Duration = Duration::from_millis(100);

// ----- encoding -----

#[derive(Default)]
struct Out(Vec<u8>);

impl Out {
    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn i32(&mut self, v: i32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn i64(&mut self, v: i64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn f32(&mut self, v: f32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn f64(&mut self, v: f64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
        self
    }
    fn str(&mut self, s: &str) -> &mut Self {
        self.bytes(s.as_bytes())
    }
}

struct In<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> In<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, at: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], PluginError> {
        let s = self
            .b
            .get(self.at..self.at + n)
            .ok_or(PluginError::Bridge("short message"))?;
        self.at += n;
        Ok(s)
    }
    fn arr<const N: usize>(&mut self) -> Result<[u8; N], PluginError> {
        self.take(N)?
            .try_into()
            .map_err(|_| PluginError::Bridge("short message"))
    }
    fn u8(&mut self) -> Result<u8, PluginError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, PluginError> {
        Ok(u32::from_le_bytes(self.arr()?))
    }
    fn i32(&mut self) -> Result<i32, PluginError> {
        Ok(i32::from_le_bytes(self.arr()?))
    }
    fn i64(&mut self) -> Result<i64, PluginError> {
        Ok(i64::from_le_bytes(self.arr()?))
    }
    fn f32(&mut self) -> Result<f32, PluginError> {
        Ok(f32::from_le_bytes(self.arr()?))
    }
    fn f64(&mut self) -> Result<f64, PluginError> {
        Ok(f64::from_le_bytes(self.arr()?))
    }
    fn bytes(&mut self) -> Result<&'a [u8], PluginError> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn string(&mut self) -> Result<String, PluginError> {
        Ok(String::from_utf8_lossy(self.bytes()?).into_owned())
    }
}

fn send(s: &mut UnixStream, msg: &[u8]) -> std::io::Result<()> {
    s.write_all(&(msg.len() as u32).to_le_bytes())?;
    s.write_all(msg)
}

fn recv(s: &mut UnixStream, buf: &mut Vec<u8>) -> std::io::Result<()> {
    let mut len = [0u8; 4];
    s.read_exact(&mut len)?;
    let n = u32::from_le_bytes(len) as usize;
    if n > MAX_MESSAGE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    buf.resize(n, 0);
    s.read_exact(buf)
}

/// Control requests (the first byte of a message).
mod op {
    pub const INIT: u8 = 0;
    pub const SET_PARAM: u8 = 1;
    pub const POLL: u8 = 2;
    pub const PARAM_TEXT: u8 = 3;
    pub const STATE: u8 = 4;
    pub const SET_STATE: u8 = 5;
    pub const PROGRAMS: u8 = 6;
    pub const SET_PROGRAM: u8 = 7;
    pub const LOAD_PRESET: u8 = 8;
    pub const PRESET: u8 = 9;
    pub const SHOW_PARAM: u8 = 10;
    pub const QUIT: u8 = 11;
}

fn encode_params(o: &mut Out, params: &[ParamInfo]) {
    o.u32(params.len() as u32);
    for p in params {
        o.u32(p.id)
            .str(&p.title)
            .str(&p.units)
            .i32(p.step_count)
            .f64(p.default)
            .i32(p.flags);
    }
}

fn decode_params(i: &mut In) -> Result<Vec<ParamInfo>, PluginError> {
    let n = i.u32()? as usize;
    let mut out = Vec::with_capacity(n.min(100_000));
    for _ in 0..n {
        out.push(ParamInfo {
            id: i.u32()?,
            title: i.string()?,
            units: i.string()?,
            step_count: i.i32()?,
            default: i.f64()?,
            flags: i.i32()?,
        });
    }
    Ok(out)
}

// ----- the child -----

/// The bridge child's `main`: connect to `socket`, load the plugin the parent names and serve it
/// until the parent goes. `load` turns a path into a module (it knows LimeShot's built-in test
/// plugins). Returns the process exit code.
pub fn child_main(socket: &Path, load: &dyn Fn(&str) -> Result<Arc<Vst3Module>, PluginError>) -> i32 {
    let connect = |kind: u8| -> std::io::Result<UnixStream> {
        let mut s = UnixStream::connect(socket)?;
        s.write_all(&[kind])?;
        Ok(s)
    };
    let (Ok(mut control), Ok(audio)) = (connect(0), connect(1)) else {
        return 2;
    };
    let mut buf = Vec::new();
    // Init: path, class, sample rate, block size.
    let (mut inst, proc) = match recv(&mut control, &mut buf)
        .map_err(|_| PluginError::Bridge("no request"))
        .and_then(|()| {
            let mut i = In::new(&buf);
            if i.u8()? != op::INIT {
                return Err(PluginError::Bridge("expected init"));
            }
            let path = i.string()?;
            let class = i.string()?;
            let sr = i.f64()?;
            let max_block = i.u32()? as usize;
            let module = load(&path)?;
            Vst3Instance::new(&module, &class, sr, max_block)
        }) {
        Ok(pair) => pair,
        Err(e) => {
            let mut o = Out::default();
            o.u8(1).str(&e.to_string());
            let _ = send(&mut control, &o.0);
            return 1;
        }
    };
    let mut o = Out::default();
    o.u8(0);
    encode_params(&mut o, inst.params());
    o.u32(inst.latency_samples());
    if send(&mut control, &o.0).is_err() {
        return 1;
    }
    let max_block = proc_max_block(&buf);
    let audio_thread = std::thread::Builder::new()
        .name("limeshot-bridge-audio".into())
        .spawn(move || serve_audio(audio, proc, max_block));
    if audio_thread.is_err() {
        return 1;
    }
    loop {
        if recv(&mut control, &mut buf).is_err() {
            return 0;
        }
        let reply = match handle(&mut inst, &buf) {
            Ok(Some(o)) => o,
            Ok(None) => return 0,
            Err(e) => {
                let mut o = Out::default();
                o.u8(1).str(&e.to_string());
                o
            }
        };
        if send(&mut control, &reply.0).is_err() {
            return 0;
        }
    }
}

fn proc_max_block(init: &[u8]) -> usize {
    let mut i = In::new(init);
    let _ = (i.u8(), i.bytes(), i.bytes(), i.f64());
    i.u32().map_or(1024, |n| n as usize).clamp(1, 1 << 16)
}

/// One control request. `None` means quit.
fn handle(inst: &mut Vst3Instance, req: &[u8]) -> Result<Option<Out>, PluginError> {
    let mut i = In::new(req);
    let mut o = Out::default();
    o.u8(0);
    match i.u8()? {
        op::SET_PARAM => inst.set_param(i.u32()?, i.f64()?),
        op::SHOW_PARAM => inst.show_param(i.u32()?, i.f64()?),
        op::POLL => {
            let restart = inst.take_restart();
            let edits = !inst.take_editor_edits().is_empty();
            o.u8(u8::from(restart || edits)).u32(inst.latency_samples());
            let ids: Vec<u32> = inst.params().iter().map(|p| p.id).collect();
            o.u32(ids.len() as u32);
            for id in ids {
                o.f64(inst.param(id));
            }
        }
        op::PARAM_TEXT => {
            let (id, v) = (i.u32()?, i.f64()?);
            o.str(&inst.param_text(id, v));
        }
        op::STATE => {
            o.bytes(&inst.state()?);
        }
        op::SET_STATE => inst.set_state(i.bytes()?)?,
        op::PROGRAMS => {
            let names = inst.program_names();
            o.u32(names.len() as u32);
            for n in &names {
                o.str(n);
            }
            o.i32(inst.program().map_or(-1, |p| p as i32));
        }
        op::SET_PROGRAM => inst.set_program(i.u32()? as usize),
        op::LOAD_PRESET => {
            let preset = VstPreset {
                class_id: i.string()?,
                component: i.bytes()?.to_vec(),
                controller: i.bytes()?.to_vec(),
            };
            inst.load_preset(&preset)?;
        }
        op::PRESET => {
            let p = inst.preset()?;
            o.str(&p.class_id).bytes(&p.component).bytes(&p.controller);
        }
        op::QUIT => return Ok(None),
        _ => return Err(PluginError::Bridge("unknown request")),
    }
    Ok(Some(o))
}

fn serve_audio(mut s: UnixStream, mut proc: crate::Vst3Processor, max_block: usize) {
    let mut req = Vec::with_capacity(64 + MAX_EVENTS * 16 + MAX_PARAMS * 16 + 8 * max_block);
    let mut audio = vec![0.0f32; 2 * max_block];
    let mut events: Vec<NoteEvent> = Vec::with_capacity(MAX_EVENTS);
    let mut reply = Out(Vec::with_capacity(16 + 8 * max_block));
    loop {
        if recv(&mut s, &mut req).is_err() {
            return;
        }
        let Ok((frames, transport, reset)) = decode_block(&req, &mut events, &mut proc, &mut audio) else {
            return;
        };
        if reset {
            proc.restart();
        }
        proc.process(&mut audio[..2 * frames], &events, &transport);
        reply.0.clear();
        reply.u32(proc.latency()).u32(frames as u32);
        for v in &audio[..2 * frames] {
            reply.f32(*v);
        }
        if send(&mut s, &reply.0).is_err() {
            return;
        }
    }
}

fn decode_block(
    req: &[u8],
    events: &mut Vec<NoteEvent>,
    proc: &mut crate::Vst3Processor,
    audio: &mut [f32],
) -> Result<(usize, Transport, bool), PluginError> {
    let mut i = In::new(req);
    let reset = i.u8()? != 0;
    let frames = (i.u32()? as usize).min(audio.len() / 2);
    let transport = Transport {
        playing: i.u8()? != 0,
        tempo: i.f64()?,
        sample_pos: i.i64()?,
        ppq_pos: i.f64()?,
        time_sig: (i.i32()?, i.i32()?),
    };
    events.clear();
    for _ in 0..i.u32()? {
        let offset = i.u32()?;
        let kind = match i.u8()? {
            0 => NoteEventKind::On {
                key: i.u8()?,
                velocity: i.f32()?,
                id: i.i32()?,
            },
            _ => {
                let key = i.u8()?;
                let _ = i.f32()?;
                NoteEventKind::Off { key, id: i.i32()? }
            }
        };
        if events.len() < events.capacity() {
            events.push(NoteEvent { offset, kind });
        }
    }
    for _ in 0..i.u32()? {
        proc.set_param_at(i.u32()?, i.u32()?, i.f64()?);
    }
    for v in &mut audio[..2 * frames] {
        *v = i.f32()?;
    }
    Ok((frames, transport, reset))
}

// ----- the parent -----

/// What both halves of a bridged plugin share.
struct Shared {
    crashed: AtomicBool,
    latency: AtomicU32,
}

/// The control side of a plugin running in a child process. Calls go to the child and wait
/// for its answer; once the child has crashed they answer with defaults.
pub struct BridgedInstance {
    shared: Arc<Shared>,
    params: Vec<ParamInfo>,
    control: Mutex<Control>,
}

struct Control {
    child: Child,
    stream: UnixStream,
    values: Vec<f64>,
    refreshed: Instant,
    texts: std::collections::HashMap<(u32, u64), String>,
    restart: bool,
    buf: Vec<u8>,
}

/// The audio side of a bridged plugin. Hand it back to the control thread to drop it.
pub struct BridgedProcessor {
    audio: UnixStream,
    shared: Arc<Shared>,
    max_block: usize,
    req: Out,
    reply: Vec<u8>,
    pending: Vec<(u32, u32, f64)>,
    reset: bool,
    /// The transport for effects, which get it separately (`Effect::set_transport`).
    transport: Transport,
}

impl Control {
    /// One request and its reply (after the status byte). Marks the plugin crashed and stops the
    /// child when the connection fails.
    fn call(&mut self, shared: &Shared, req: &Out, what: &'static str) -> Result<In<'_>, PluginError> {
        if shared.crashed.load(Ordering::Relaxed) {
            return Err(PluginError::Bridge("reach the plugin: its process has crashed"));
        }
        let ok = send(&mut self.stream, &req.0).and_then(|()| recv(&mut self.stream, &mut self.buf));
        if let Err(e) = ok {
            shared.crashed.store(true, Ordering::Relaxed);
            let _ = self.child.kill();
            return Err(bridge_io(what, e));
        }
        let mut i = In::new(&self.buf);
        match i.u8()? {
            0 => Ok(i),
            _ => Err(PluginError::Remote(i.string().unwrap_or_default())),
        }
    }

    /// Fetch parameter values, latency and the restart flag (at most every 100 ms unless `now`).
    fn refresh(&mut self, shared: &Shared, now: bool) {
        if !now && self.refreshed.elapsed() < REFRESH {
            return;
        }
        self.refreshed = Instant::now();
        let mut o = Out::default();
        o.u8(op::POLL);
        let parsed = self.call(shared, &o, "poll").and_then(|mut i| {
            let restart = i.u8()? != 0;
            let latency = i.u32()?;
            let n = i.u32()? as usize;
            let mut v = Vec::with_capacity(n.min(100_000));
            for _ in 0..n {
                v.push(i.f64()?);
            }
            Ok((restart, latency, v))
        });
        if let Ok((restart, latency, values)) = parsed {
            self.restart |= restart;
            shared.latency.store(latency, Ordering::Relaxed);
            if values.len() == self.values.len() {
                self.values = values;
            }
        }
    }
}

impl BridgedInstance {
    /// Start `exe` as a bridge and load class `class_id` of the bundle at `path` in it.
    pub fn spawn(
        exe: &Path,
        path: &str,
        class_id: &str,
        sample_rate: f64,
        max_block: usize,
    ) -> Result<(BridgedInstance, BridgedProcessor), PluginError> {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let socket = std::env::temp_dir().join(format!(
            "limeshot-bridge-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).map_err(|e| bridge_io("listen", e))?;
        let child = Command::new(exe)
            .arg(BRIDGE_FLAG)
            .arg(&socket)
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| bridge_io("start the bridge", e));
        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&socket);
                return Err(e);
            }
        };
        let accepted = accept_two(&listener, &mut child);
        let _ = std::fs::remove_file(&socket);
        let fail = |mut child: Child, e: PluginError| {
            let _ = child.kill();
            let _ = child.wait();
            e
        };
        let (mut stream, audio) = match accepted {
            Ok(pair) => pair,
            Err(e) => return Err(fail(child, e)),
        };
        let max_block = max_block.max(1);
        let mut o = Out::default();
        o.u8(op::INIT)
            .str(path)
            .str(class_id)
            .f64(sample_rate)
            .u32(max_block as u32);
        let mut buf = Vec::new();
        let _ = stream.set_read_timeout(Some(START_TIMEOUT));
        if let Err(e) = send(&mut stream, &o.0).and_then(|()| recv(&mut stream, &mut buf)) {
            return Err(fail(child, bridge_io("load the plugin", e)));
        }
        let (params, latency) = match parse_init(&buf) {
            Ok(v) => v,
            Err(e) => return Err(fail(child, e)),
        };
        let _ = stream.set_read_timeout(Some(CONTROL_TIMEOUT));
        let _ = stream.set_write_timeout(Some(CONTROL_TIMEOUT));
        let _ = audio.set_read_timeout(Some(AUDIO_TIMEOUT));
        let _ = audio.set_write_timeout(Some(AUDIO_TIMEOUT));
        let shared = Arc::new(Shared {
            crashed: AtomicBool::new(false),
            latency: AtomicU32::new(latency),
        });
        let mut control = Control {
            child,
            stream,
            values: params.iter().map(|p| p.default).collect(),
            refreshed: Instant::now(),
            texts: Default::default(),
            restart: false,
            buf,
        };
        control.refresh(&shared, true);
        let inst = BridgedInstance {
            shared: shared.clone(),
            params,
            control: Mutex::new(control),
        };
        let proc = BridgedProcessor {
            audio,
            shared,
            max_block,
            req: Out(Vec::with_capacity(
                64 + MAX_EVENTS * 16 + MAX_PARAMS * 16 + 8 * max_block,
            )),
            reply: Vec::with_capacity(16 + 8 * max_block),
            pending: Vec::with_capacity(MAX_PARAMS),
            reset: false,
            transport: Transport::default(),
        };
        Ok((inst, proc))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Control> {
        self.control.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Whether the plugin's process crashed, hung or quit. A hung child is stopped for good.
    pub fn crashed(&self) -> bool {
        let mut c = self.lock();
        if !matches!(c.child.try_wait(), Ok(None)) {
            self.shared.crashed.store(true, Ordering::Relaxed);
        }
        let crashed = self.shared.crashed.load(Ordering::Relaxed);
        if crashed {
            let _ = c.child.kill();
        }
        crashed
    }

    /// The child's process id (tests stop it to stand in for a crash).
    pub fn process_id(&self) -> u32 {
        self.lock().child.id()
    }

    pub fn params(&self) -> &[ParamInfo] {
        &self.params
    }

    pub fn param(&self, id: u32) -> f64 {
        let c = self.lock();
        self.params
            .iter()
            .position(|p| p.id == id)
            .and_then(|i| c.values.get(i))
            .copied()
            .unwrap_or(0.0)
    }

    fn send_value(&self, op: u8, id: u32, value: f64) {
        let mut c = self.lock();
        if let Some(i) = self.params.iter().position(|p| p.id == id) {
            c.values[i] = value;
        }
        let mut o = Out::default();
        o.u8(op).u32(id).f64(value);
        let _ = c.call(&self.shared, &o, "set a parameter");
    }

    pub fn set_param(&self, id: u32, value: f64) {
        let value = if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 };
        self.send_value(op::SET_PARAM, id, value);
    }

    pub fn show_param(&self, id: u32, value: f64) {
        if (self.param(id) - value).abs() > 1e-6 {
            self.send_value(op::SHOW_PARAM, id, value.clamp(0.0, 1.0));
        }
    }

    pub fn param_text(&self, id: u32, value: f64) -> String {
        let key = (id, value.to_bits());
        let mut c = self.lock();
        if let Some(t) = c.texts.get(&key) {
            return t.clone();
        }
        let mut o = Out::default();
        o.u8(op::PARAM_TEXT).u32(id).f64(value);
        let text = c
            .call(&self.shared, &o, "show a value")
            .and_then(|mut i| i.string())
            .unwrap_or_else(|_| format!("{value:.3}"));
        if c.texts.len() > 4096 {
            c.texts.clear();
        }
        c.texts.insert(key, text.clone());
        text
    }

    /// Whether the plugin changed its own parameters or settings since the last call.
    pub fn take_restart(&self) -> bool {
        let mut c = self.lock();
        c.refresh(&self.shared, false);
        std::mem::take(&mut c.restart)
    }

    pub fn latency_samples(&self) -> u32 {
        self.shared.latency.load(Ordering::Relaxed)
    }

    pub fn state(&self) -> Result<Vec<u8>, PluginError> {
        let mut o = Out::default();
        o.u8(op::STATE);
        let mut c = self.lock();
        Ok(c.call(&self.shared, &o, "save its state")?.bytes()?.to_vec())
    }

    pub fn set_state(&self, blob: &[u8]) -> Result<(), PluginError> {
        let mut o = Out::default();
        o.u8(op::SET_STATE).bytes(blob);
        let mut c = self.lock();
        c.call(&self.shared, &o, "load its state")?;
        c.refresh(&self.shared, true);
        Ok(())
    }

    /// The factory program names and the current program.
    pub fn programs(&self) -> (Vec<String>, Option<usize>) {
        let mut o = Out::default();
        o.u8(op::PROGRAMS);
        let mut c = self.lock();
        let parsed = c.call(&self.shared, &o, "list programs").and_then(|mut i| {
            let n = i.u32()? as usize;
            let mut names = Vec::with_capacity(n.min(10_000));
            for _ in 0..n {
                names.push(i.string()?);
            }
            Ok((names, usize::try_from(i.i32()?).ok()))
        });
        parsed.unwrap_or_default()
    }

    pub fn set_program(&self, index: usize) {
        let mut o = Out::default();
        o.u8(op::SET_PROGRAM).u32(index as u32);
        let mut c = self.lock();
        let _ = c.call(&self.shared, &o, "select a program");
        c.refresh(&self.shared, true);
    }

    pub fn load_preset(&self, preset: &VstPreset) -> Result<(), PluginError> {
        let mut o = Out::default();
        o.u8(op::LOAD_PRESET)
            .str(&preset.class_id)
            .bytes(&preset.component)
            .bytes(&preset.controller);
        let mut c = self.lock();
        c.call(&self.shared, &o, "load the preset")?;
        c.refresh(&self.shared, true);
        Ok(())
    }

    pub fn preset(&self) -> Result<VstPreset, PluginError> {
        let mut o = Out::default();
        o.u8(op::PRESET);
        let mut c = self.lock();
        let mut i = c.call(&self.shared, &o, "save a preset")?;
        Ok(VstPreset {
            class_id: i.string()?,
            component: i.bytes()?.to_vec(),
            controller: i.bytes()?.to_vec(),
        })
    }
}

impl Drop for BridgedInstance {
    fn drop(&mut self) {
        let crashed = self.shared.crashed.load(Ordering::Relaxed);
        let c = self.control.get_mut().unwrap_or_else(|p| p.into_inner());
        if !crashed {
            let mut o = Out::default();
            o.u8(op::QUIT);
            let _ = send(&mut c.stream, &o.0);
        }
        let _ = c.stream.shutdown(std::net::Shutdown::Both);
        // Give the child a moment to unload the plugin cleanly, then make sure it is gone.
        let until = Instant::now() + Duration::from_millis(500);
        while Instant::now() < until && matches!(c.child.try_wait(), Ok(None)) {
            std::thread::sleep(Duration::from_millis(5));
        }
        let _ = c.child.kill();
        let _ = c.child.wait();
    }
}

fn parse_init(buf: &[u8]) -> Result<(Vec<ParamInfo>, u32), PluginError> {
    let mut i = In::new(buf);
    match i.u8()? {
        0 => Ok((decode_params(&mut i)?, i.u32()?)),
        _ => Err(PluginError::Remote(i.string().unwrap_or_default())),
    }
}

fn bridge_io(what: &'static str, e: std::io::Error) -> PluginError {
    log::warn!("plugin bridge: could not {what}: {e}");
    PluginError::Bridge(what)
}

/// Accept the child's control and audio connections (either order), giving up when the child
/// exits or after [`START_TIMEOUT`].
fn accept_two(listener: &UnixListener, child: &mut Child) -> Result<(UnixStream, UnixStream), PluginError> {
    listener.set_nonblocking(true).map_err(|e| bridge_io("listen", e))?;
    let deadline = Instant::now() + START_TIMEOUT;
    let (mut control, mut audio) = (None, None);
    while control.is_none() || audio.is_none() {
        match listener.accept() {
            Ok((mut s, _)) => {
                s.set_nonblocking(false).map_err(|e| bridge_io("connect", e))?;
                let _ = s.set_read_timeout(Some(START_TIMEOUT));
                let mut kind = [0u8];
                s.read_exact(&mut kind).map_err(|e| bridge_io("connect", e))?;
                if kind[0] == 0 {
                    control = Some(s);
                } else {
                    audio = Some(s);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() > deadline || !matches!(child.try_wait(), Ok(None)) {
                    return Err(PluginError::Bridge("start the plugin's process"));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(e) => return Err(bridge_io("connect", e)),
        }
    }
    match (control, audio) {
        (Some(c), Some(a)) => Ok((c, a)),
        _ => Err(PluginError::Bridge("connect")),
    }
}

impl BridgedProcessor {
    pub fn crashed(&self) -> bool {
        self.shared.crashed.load(Ordering::Relaxed)
    }

    pub fn latency(&self) -> u32 {
        self.shared.latency.load(Ordering::Relaxed)
    }

    /// Queue an automation point for the next `process` call (see
    /// [`crate::Vst3Processor::set_param_at`]).
    pub fn set_param_at(&mut self, id: u32, offset: u32, value: f64) {
        if value.is_finite() && self.pending.len() < self.pending.capacity() {
            self.pending.push((id, offset, value.clamp(0.0, 1.0)));
        }
    }

    /// Clear the plugin's tails at the next block.
    pub fn restart(&mut self) {
        self.reset = true;
    }

    /// Process `buf` (interleaved stereo) in place in the child. After a crash, or when the
    /// child does not answer in time, the output is silence.
    pub fn process(&mut self, buf: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        let frames = buf.len() / 2;
        let last = frames.saturating_sub(1) as u32;
        let mut start = 0;
        let mut t = *transport;
        while start < frames {
            let n = (frames - start).min(self.max_block);
            let chunk = &mut buf[2 * start..2 * (start + n)];
            if self.crashed() || self.block(chunk, start as u32, notes, &t, last).is_err() {
                self.shared.crashed.store(true, Ordering::Relaxed);
                chunk.fill(0.0);
            }
            start += n;
            t.sample_pos += n as i64;
        }
        self.pending.clear();
    }

    fn block(
        &mut self,
        chunk: &mut [f32],
        first: u32,
        notes: &[NoteEvent],
        t: &Transport,
        last: u32,
    ) -> std::io::Result<()> {
        let n = chunk.len() / 2;
        let end = first + n as u32;
        let o = &mut self.req;
        o.0.clear();
        o.u8(u8::from(std::mem::take(&mut self.reset))).u32(n as u32);
        o.u8(u8::from(t.playing))
            .f64(t.tempo)
            .i64(t.sample_pos)
            .f64(t.ppq_pos)
            .i32(t.time_sig.0)
            .i32(t.time_sig.1);
        let inside = |off: u32| off >= first && off < end;
        let events = notes.iter().filter(|e| inside(e.offset)).take(MAX_EVENTS);
        o.u32(events.clone().count() as u32);
        for e in events {
            o.u32(e.offset - first);
            match e.kind {
                NoteEventKind::On { key, velocity, id } => o.u8(0).u8(key).f32(velocity).i32(id),
                NoteEventKind::Off { key, id } => o.u8(1).u8(key).f32(0.0).i32(id),
            };
        }
        let params = self.pending.iter().filter(|p| inside(p.1.min(last)));
        o.u32(params.clone().count() as u32);
        for &(id, offset, v) in params {
            o.u32(id).u32(offset.min(last) - first).f64(v);
        }
        for s in chunk.iter() {
            o.f32(*s);
        }
        send(&mut self.audio, &o.0)?;
        recv(&mut self.audio, &mut self.reply)?;
        let mut i = In::new(&self.reply);
        let bad = |_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad reply");
        self.shared.latency.store(i.u32().map_err(bad)?, Ordering::Relaxed);
        if i.u32().map_err(bad)? as usize != n {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad reply"));
        }
        for s in chunk.iter_mut() {
            *s = i.f32().map_err(bad)?;
        }
        Ok(())
    }
}

impl limeshot_dsp::effects::Effect for BridgedProcessor {
    fn set_params(&mut self, _params: &[f32]) {}
    fn process(&mut self, buf: &mut [f32]) {
        let t = self.transport;
        BridgedProcessor::process(self, buf, &[], &t);
    }
    fn reset(&mut self) {
        self.restart();
    }
    fn set_transport(&mut self, transport: &Transport) {
        self.transport = *transport;
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.set_param_at(id, offset, value);
    }
    fn latency(&self) -> u32 {
        BridgedProcessor::latency(self)
    }
}

impl limeshot_dsp::Instrument for BridgedProcessor {
    fn process(&mut self, out: &mut [f32], notes: &[NoteEvent], transport: &Transport) {
        out.fill(0.0);
        BridgedProcessor::process(self, out, notes, transport);
    }
    fn reset(&mut self) {
        self.restart();
    }
    fn set_plugin_param(&mut self, id: u32, offset: u32, value: f64) {
        self.set_param_at(id, offset, value);
    }
    fn latency(&self) -> u32 {
        BridgedProcessor::latency(self)
    }
}

/// The socket path argument of a bridge child's command line, if `args` (without the program
/// name) start one.
pub fn socket_arg(args: &[String]) -> Option<PathBuf> {
    match args {
        [flag, socket] if flag == BRIDGE_FLAG => Some(PathBuf::from(socket)),
        _ => None,
    }
}
