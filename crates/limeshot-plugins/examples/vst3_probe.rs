//! Load one VST3 bundle on the main thread, instantiate each class and process a few blocks.
//! `cargo run -p limeshot-plugins --example vst3_probe -- /Library/Audio/Plug-Ins/VST3/X.vst3`

use limeshot_plugins::{NoteEvent, NoteEventKind, Transport, Vst3Instance, Vst3Module};

fn main() {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: vst3_probe <bundle.vst3>");
        std::process::exit(2);
    };
    let path = std::path::PathBuf::from(path);
    let module = match Vst3Module::load(&path) {
        Ok(m) => m,
        Err(e) => {
            println!("{}: {e}", path.display());
            std::process::exit(1);
        }
    };
    for c in module.classes() {
        match Vst3Instance::new(&module, &c.class_id, 48_000.0, 512) {
            Ok((inst, mut proc)) => {
                let mut buf = vec![0.1f32; 1024];
                let on = [NoteEvent {
                    offset: 0,
                    kind: NoteEventKind::On {
                        key: 60,
                        velocity: 0.8,
                        id: 1,
                    },
                }];
                let mut peak = 0.0f32;
                for i in 0..40 {
                    proc.process(&mut buf, if i == 0 { &on } else { &[] }, &Transport::default());
                    peak = buf.iter().fold(peak, |m, s| m.max(s.abs()));
                }
                let state = inst.state().map(|s| s.len()).unwrap_or(0);
                let programs = inst.program_names();
                println!(
                    "{} [{:?}] {} params, peak {peak:.3}, state {state} B, latency {}, {} programs{}",
                    c.name,
                    c.kind,
                    inst.params().len(),
                    inst.latency_samples(),
                    programs.len(),
                    programs.iter().take(4).map(|p| format!(" [{p}]")).collect::<String>()
                );
            }
            Err(e) => println!("{}: {e}", c.name),
        }
    }
}
