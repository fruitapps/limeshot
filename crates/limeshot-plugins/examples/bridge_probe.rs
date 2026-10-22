//! Run one VST3 bundle's classes in a plugin bridge and process a few blocks.
//! `cargo run -p limeshot-plugins --example bridge_probe -- <bridge exe> /Library/Audio/Plug-Ins/VST3/X.vst3`

fn main() {
    #[cfg(unix)]
    {
        use limeshot_plugins::bridge::BridgedInstance;
        use limeshot_plugins::{NoteEvent, NoteEventKind, Transport, Vst3Module};
        let args: Vec<String> = std::env::args().skip(1).collect();
        let [exe, bundle] = args.as_slice() else {
            eprintln!("usage: bridge_probe <bridge exe> <bundle.vst3>");
            std::process::exit(2);
        };
        let classes = match Vst3Module::load(std::path::Path::new(bundle)) {
            Ok(m) => m.classes(),
            Err(e) => {
                println!("{bundle}: {e}");
                std::process::exit(1);
            }
        };
        for c in classes {
            let t = std::time::Instant::now();
            match BridgedInstance::spawn(std::path::Path::new(exe), bundle, &c.class_id, 48_000.0, 512) {
                Ok((inst, mut proc)) => {
                    let started = t.elapsed();
                    let on = [NoteEvent {
                        offset: 0,
                        kind: NoteEventKind::On {
                            key: 60,
                            velocity: 0.8,
                            id: 1,
                        },
                    }];
                    let mut peak = 0.0f32;
                    let t = std::time::Instant::now();
                    for i in 0..100 {
                        let mut buf = vec![0.1f32; 1024];
                        proc.process(&mut buf, if i == 0 { &on } else { &[] }, &Transport::default());
                        peak = buf.iter().fold(peak, |m, s| m.max(s.abs()));
                    }
                    let per_block = t.elapsed() / 100;
                    let state = inst.state().map(|s| s.len()).unwrap_or(0);
                    println!(
                        "{} [{:?}] started in {started:?}, {} params, peak {peak:.3}, state {state} B, latency {}, {per_block:?} per block, crashed {}",
                        c.name,
                        c.kind,
                        inst.params().len(),
                        inst.latency_samples(),
                        inst.crashed()
                    );
                }
                Err(e) => println!("{}: {e}", c.name),
            }
        }
    }
}
