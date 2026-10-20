//! `limeshot-cli`: headless LimeShot for scripting, tests and agents. No network service.
//!
//! ```text
//! limeshot-cli info   <project.lime>
//! limeshot-cli render <project.lime> <out.wav> [--pattern] [--rate 48000] [--bits 16|24|32f]
//! limeshot-cli run    <commands.json> [--project in.lime] [--save out.lime] [--render out.wav]
//! limeshot-cli demo   <out.lime> [--render out.wav]
//! ```
//!
//! `run` applies a JSON array of `limeshot_core::Command` values (the same typed commands the UI
//! executes), e.g. `[{"SetTempo": 128000}, {"AddPattern": {"name": "Verse"}}]`.

use limeshot_audio::assets::SampleCache;
use limeshot_core::{Command, Document, Project};
use limeshot_engine::{BitDepth, ExportRange, RenderSettings};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  limeshot-cli info <project.lime>\n  limeshot-cli render <project.lime> <out.wav> [--pattern] [--rate N] [--bits 16|24|32f]\n  limeshot-cli run <commands.json> [--project in.lime] [--save out.lime] [--render out.wav]\n  limeshot-cli demo <out.lime> [--render out.wav]"
    );
    ExitCode::from(2)
}

struct Opts {
    positional: Vec<String>,
    pattern: bool,
    rate: u32,
    depth: BitDepth,
    project: Option<PathBuf>,
    save: Option<PathBuf>,
    render: Option<PathBuf>,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts { positional: Vec::new(), pattern: false, rate: 44_100, depth: BitDepth::Int24, project: None, save: None, render: None };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--pattern" => o.pattern = true,
            "--rate" => o.rate = val()?.parse().map_err(|_| "bad --rate".to_string())?,
            "--bits" => {
                o.depth = match val()?.as_str() {
                    "16" => BitDepth::Int16,
                    "24" => BitDepth::Int24,
                    "32f" | "32" => BitDepth::Float32,
                    other => return Err(format!("unknown bit depth {other}")),
                }
            }
            "--project" => o.project = Some(val()?.into()),
            "--save" => o.save = Some(val()?.into()),
            "--render" => o.render = Some(val()?.into()),
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            s => o.positional.push(s.to_string()),
        }
    }
    Ok(o)
}

/// Load a project with absolute sample paths (as the app keeps them in memory).
fn load(path: &Path) -> Result<Project, String> {
    let mut p = limeshot_project::load(path).map_err(|e| e.to_string())?;
    let dir = path.parent();
    for c in &mut p.channels {
        if let limeshot_core::Instrument::Sampler(s) = &mut c.instrument
            && let Some(r) = &mut s.sample
        {
            r.path = limeshot_project::resolve_sample_path(&r.path, dir).to_string_lossy().into_owned();
        }
    }
    Ok(p)
}

fn render(project: &Project, out: &Path, o: &Opts) -> Result<(), String> {
    let mut cache = SampleCache::new();
    for r in project.sample_refs() {
        if let Err(e) = cache.load_blocking(&r.path) {
            eprintln!("warning: {e}");
        }
    }
    let pattern = project.patterns.first().map(|p| p.id).ok_or("project has no pattern")?;
    let snap = Arc::new(limeshot_sequencer::compile(project, pattern, &cache));
    let range = if o.pattern { ExportRange::Pattern } else { ExportRange::Song };
    let settings = RenderSettings { sample_rate: o.rate, range, ..RenderSettings::default() };
    let rep = limeshot_engine::export_wav(snap, &settings, o.depth, out).map_err(|e| e.to_string())?;
    println!(
        "rendered {} — {:.2} s ({} frames at {} Hz), peak {:.1} dBFS",
        out.display(),
        rep.seconds(),
        rep.frames,
        rep.sample_rate,
        limeshot_dsp::gain_to_db(rep.peak)
    );
    Ok(())
}

fn info(p: &Project) {
    let sig = p.time_signature;
    println!("title:      {}", p.title);
    println!("tempo:      {} BPM, {}", p.tempo, sig);
    println!("channels:   {}", p.channels.len());
    for c in &p.channels {
        println!("  - {} ({}) → mixer {}", c.name, c.instrument.kind_label(), c.mixer_track);
    }
    println!("patterns:   {}", p.patterns.len());
    for pat in &p.patterns {
        println!("  - {}: {} notes, {} bars", pat.name, pat.notes.len(), pat.length(sig) / sig.ticks_per_bar());
    }
    println!("clips:      {} (song {} bars)", p.playlist.clips.len(), p.playlist.song_length(sig) / sig.ticks_per_bar());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else { return usage() };
    let o = match parse(rest) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return usage();
        }
    };
    let result = match cmd.as_str() {
        "info" => match o.positional.first() {
            Some(p) => load(Path::new(p)).map(|p| info(&p)),
            None => return usage(),
        },
        "render" => match (o.positional.first(), o.positional.get(1)) {
            (Some(p), Some(out)) => load(Path::new(p)).and_then(|proj| render(&proj, Path::new(out), &o)),
            _ => return usage(),
        },
        "run" => {
            let Some(script) = o.positional.first() else { return usage() };
            (|| {
                let text = std::fs::read_to_string(script).map_err(|e| format!("{script}: {e}"))?;
                let cmds: Vec<Command> = serde_json::from_str(&text).map_err(|e| format!("{script}: {e}"))?;
                let mut doc = Document::new(match &o.project {
                    Some(p) => load(p)?,
                    None => Project::new(),
                });
                for (i, c) in cmds.into_iter().enumerate() {
                    let label = c.label();
                    let out = doc.execute(c).map_err(|e| format!("command {} ({label}): {e}", i + 1))?;
                    if let Some(id) = out.0 {
                        println!("{}: {label} → id {id}", i + 1);
                    }
                }
                if let Some(s) = &o.save {
                    limeshot_project::save(doc.project(), s).map_err(|e| e.to_string())?;
                    println!("saved {}", s.display());
                }
                if let Some(r) = &o.render {
                    render(doc.project(), r, &o)?;
                }
                Ok(())
            })()
        }
        "demo" => {
            let Some(out) = o.positional.first() else { return usage() };
            (|| {
                let samples = limeshot_app::bundled_samples_dir().ok_or("bundled samples not found")?;
                let doc = limeshot_app::demo::build(&samples)?;
                limeshot_project::save(doc.project(), Path::new(out)).map_err(|e| e.to_string())?;
                println!("saved {out}");
                if let Some(r) = &o.render {
                    render(doc.project(), r, &o)?;
                }
                Ok(())
            })()
        }
        _ => return usage(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
