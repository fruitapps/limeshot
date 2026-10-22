//! Plugins in a bridge process: they work as in-process plugins do, and a crash or hang of the
//! bridge leaves the host running with the plugin silent.
#![cfg(unix)]

use std::path::Path;
use std::time::{Duration, Instant};

use limeshot_plugins::bridge::{AUDIO_TIMEOUT, BridgedInstance, BridgedProcessor};
use limeshot_plugins::{NoteEvent, NoteEventKind, Transport, testing};

const EXE: &str = env!("CARGO_BIN_EXE_limeshot-plugin-bridge");

fn spawn(class: &str) -> (BridgedInstance, BridgedProcessor) {
    BridgedInstance::spawn(Path::new(EXE), testing::BUILTIN_PATH, class, 48_000.0, 64).expect("bridge")
}

#[test]
fn a_bridged_plugin_processes_and_keeps_its_parameters_and_state() {
    let (inst, mut proc) = spawn(&testing::gain_class());
    assert_eq!(inst.params().len(), 1);
    assert_eq!(inst.params()[0].title, "Gain");
    let mut buf = vec![0.5f32; 2 * 200];
    proc.process(&mut buf, &[], &Transport::default());
    assert!(buf.iter().all(|&s| s == 0.5), "unity gain through the bridge");

    inst.set_param(0, 0.5);
    assert_eq!(inst.param(0), 0.5);
    assert_eq!(inst.param_text(0, 0.25), "25%");
    proc.process(&mut buf, &[], &Transport::default());
    assert!(buf.iter().all(|&s| s == 0.25), "half gain");
    // Automation points reach the processor in the child too.
    proc.set_param_at(0, 10, 1.0);
    buf.fill(0.5);
    proc.process(&mut buf, &[], &Transport::default());
    assert_eq!(buf[2 * 199], 0.5);

    let state = inst.state().expect("state");
    let (other, _p) = spawn(&testing::gain_class());
    other.set_state(&state).expect("set state");
    assert_eq!(
        other.param(0),
        1.0,
        "the processor's automated value travels with the state"
    );
    assert!(!inst.crashed());
}

#[test]
fn a_bridged_instrument_plays_notes_and_lists_its_programs() {
    let (inst, mut proc) = spawn(&testing::tone_class());
    let on = [NoteEvent {
        offset: 100,
        kind: NoteEventKind::On {
            key: 69,
            velocity: 1.0,
            id: 1,
        },
    }];
    let mut buf = vec![0.0f32; 2 * 256];
    proc.process(&mut buf, &on, &Transport::default());
    assert!(buf[..200].iter().all(|&s| s == 0.0), "silent before the note");
    assert!(buf[200..].iter().any(|&s| s.abs() > 0.1), "sounds from its offset");
    let p = limeshot_plugins::PluginInstance::Bridged(inst);
    assert_eq!(p.program_names(), ["Medium", "Quiet", "Loud"]);
    assert_eq!(p.program(), Some(0));
}

fn signal(pid: u32, sig: &str) {
    let ok = std::process::Command::new("kill")
        .arg(sig)
        .arg(pid.to_string())
        .status()
        .expect("kill")
        .success();
    assert!(ok);
}

#[test]
fn a_crashed_bridge_leaves_silence_and_the_host_running() {
    let (inst, mut proc) = spawn(&testing::gain_class());
    let mut buf = vec![0.5f32; 2 * 64];
    proc.process(&mut buf, &[], &Transport::default());
    assert_eq!(buf[0], 0.5);
    signal(inst.process_id(), "-KILL");
    std::thread::sleep(Duration::from_millis(50));
    let mut buf = vec![0.5f32; 2 * 64];
    let t = Instant::now();
    proc.process(&mut buf, &[], &Transport::default());
    assert!(t.elapsed() < AUDIO_TIMEOUT, "a dead process is noticed at once");
    assert!(buf.iter().all(|&s| s == 0.0), "silence after the crash");
    assert!(proc.crashed() && inst.crashed());
    assert!(inst.state().is_err());
    assert_eq!(inst.param(0), 1.0, "the last known value");
}

#[test]
fn a_hung_bridge_times_out() {
    let (inst, mut proc) = spawn(&testing::gain_class());
    signal(inst.process_id(), "-STOP");
    let mut buf = vec![0.5f32; 2 * 64];
    let t = Instant::now();
    proc.process(&mut buf, &[], &Transport::default());
    assert!(t.elapsed() < AUDIO_TIMEOUT * 3, "{:?}", t.elapsed());
    assert!(buf.iter().all(|&s| s == 0.0));
    assert!(inst.crashed(), "the hung process is stopped for good");
    // Later blocks do not wait again.
    let t = Instant::now();
    proc.process(&mut buf, &[], &Transport::default());
    assert!(t.elapsed() < Duration::from_millis(50));
    drop(inst);
}
