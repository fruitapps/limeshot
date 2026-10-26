//! Hostile-input robustness: `.lime` files, MIDI and FL Studio projects are untrusted input.
//! Every numeric field of a real project is replaced by extreme values, and file bytes are
//! corrupted; loading, compiling and rendering must report errors or repair the data, never
//! panic. Failures are collected so one run lists every offending field.

use limeshot_audio::assets::SampleCache;
use limeshot_engine::{ExportRange, RenderSettings, render_offline};
use serde_json::Value;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;

/// Deterministic xorshift so failures reproduce.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn numeric_paths(v: &Value, here: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match v {
        Value::Number(_) => out.push(here.clone()),
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                here.push(i.to_string());
                numeric_paths(x, here, out);
                here.pop();
            }
        }
        Value::Object(o) => {
            for (k, x) in o {
                here.push(k.clone());
                numeric_paths(x, here, out);
                here.pop();
            }
        }
        _ => {}
    }
}

fn set(v: &mut Value, path: &[String], new: Value) {
    let mut cur = v;
    for key in path {
        cur = match cur {
            Value::Array(a) => &mut a[key.parse::<usize>().unwrap()],
            Value::Object(o) => o.get_mut(key).unwrap(),
            _ => return,
        };
    }
    *cur = new;
}

/// Load, then (if it loads) compile and render a short stretch.
fn exercise(text: &str, samples: &Path) {
    let Ok(project) = limeshot_project::from_json(text, Path::new("hostile.lime")) else {
        return;
    };
    let mut cache = SampleCache::new();
    cache.set_project_dir(Some(samples.to_path_buf()));
    for r in project.sample_refs() {
        let _ = cache.load_blocking(&r.path);
    }
    let Some(pattern) = project.patterns.first().map(|p| p.id) else {
        return;
    };
    let snap = Arc::new(limeshot_sequencer::compile(&project, pattern, &cache));
    // A hostile length must not be rendered in full, but must not crash the compile either.
    if snap.pattern.length > 960 * 16 {
        return;
    }
    let settings = RenderSettings {
        sample_rate: 22_050,
        range: ExportRange::Pattern,
        max_tail_secs: 0.1,
        ..RenderSettings::default()
    };
    let _ = render_offline(snap, &settings);
}

#[test]
fn extreme_numbers_in_a_project_never_panic() {
    let samples = limeshot_app::bundled_samples_dir().expect("bundled samples");
    let doc = limeshot_app::demo::build(&samples).unwrap();
    let json = limeshot_project::to_json(doc.project(), Some(&samples)).unwrap();
    let base: Value = serde_json::from_str(&json).unwrap();
    let mut paths = Vec::new();
    numeric_paths(&base["project"], &mut vec!["project".into()], &mut paths);
    // `numeric_paths` was seeded with the "project" key, so paths are absolute from the root.
    let extremes: [Value; 7] = [
        serde_json::json!(0),
        serde_json::json!(-1),
        serde_json::json!(4_294_967_295u64),
        serde_json::json!(u64::MAX),
        serde_json::json!(i64::MIN),
        serde_json::json!(1e30),
        serde_json::json!(-1e30),
    ];
    let mut failures = Vec::new();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    // Every field with every extreme loads; compile + render runs on a random subset to bound time.
    for path in &paths {
        for (i, e) in extremes.iter().enumerate() {
            let mut v = base.clone();
            set(&mut v, path, e.clone());
            let text = v.to_string();
            let run = rng.next().is_multiple_of(4);
            let r = catch_unwind(AssertUnwindSafe(|| {
                if run {
                    exercise(&text, &samples);
                } else {
                    let _ = limeshot_project::from_json(&text, Path::new("hostile.lime"));
                }
            }));
            if r.is_err() {
                failures.push(format!("{} = extreme #{i}", path.join(".")));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} panics:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn corrupted_midi_and_flp_bytes_never_panic() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../limeshot-project/tests/fixtures/flp");
    let mut inputs: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "flp") {
            inputs.push((p.display().to_string(), std::fs::read(&p).unwrap()));
        }
    }
    let samples = limeshot_app::bundled_samples_dir().unwrap();
    let doc = limeshot_app::demo::build(&samples).unwrap();
    let pattern = doc.project().patterns[0].id;
    let snap = limeshot_sequencer::compile(doc.project(), pattern, &SampleCache::new());
    let midi = limeshot_project::midi::write(&limeshot_project::midi::from_program(doc.project(), &snap.song));
    inputs.push(("demo.mid".into(), midi));

    let mut rng = Rng(12345);
    let mut failures = Vec::new();
    for (name, bytes) in &inputs {
        for round in 0..400 {
            let mut b = bytes.clone();
            match round % 3 {
                0 => {
                    let n = 1 + (rng.next() % 8) as usize;
                    for _ in 0..n {
                        if b.is_empty() {
                            break;
                        }
                        let i = (rng.next() as usize) % b.len();
                        b[i] = rng.next() as u8;
                    }
                }
                1 => b.truncate((rng.next() as usize) % (b.len() + 1)),
                _ => {
                    // Overwrite a 4-byte run with 0xFF: hostile lengths and counts.
                    if b.len() >= 4 {
                        let i = (rng.next() as usize) % (b.len() - 3);
                        b[i..i + 4].fill(0xFF);
                    }
                }
            }
            let r = catch_unwind(AssertUnwindSafe(|| {
                if name.ends_with(".mid") {
                    if let Ok(f) = limeshot_project::midi::read(&b) {
                        let _ = limeshot_project::midi::import_command(&f, "x", true);
                    }
                } else if let Ok(f) = limeshot_project::flp::parse(&b) {
                    let (mut p, _) = limeshot_project::flp::to_project(&f, None);
                    p.sanitize();
                }
            }));
            if r.is_err() {
                failures.push(format!("{name} round {round}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} panics:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
