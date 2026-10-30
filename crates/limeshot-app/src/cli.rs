//! `limeshot-cli`: headless LimeShot for scripting, tests and agents. No network service.
//!
//! ```text
//! limeshot-cli info   <project.lime>
//! limeshot-cli render <project.lime> <out.wav|out.mp3> [--pattern] [--rate 48000] [--bits 16|24|32f] [--bitrate 320]
//!                     [--dither] [--split]   (WAV: TPDF dither; one more WAV per sounding mixer insert)
//! limeshot-cli run    <commands.json> [--project in.lime] [--save out.lime] [--render out.wav]
//! limeshot-cli demo   <out.lime> [--render out.wav]
//! limeshot-cli play   <project.lime> [--pattern] [--seconds 5]   (realtime, through the audio device)
//! ```
//!
//! `run` applies a JSON array of `limeshot_core::Command` values (the same typed commands the UI
//! executes), e.g. `[{"SetTempo": 128000}, {"AddPattern": {"name": "Verse"}}]`.

use limeshot_audio::assets::SampleCache;
use limeshot_core::{Command, Document, Project};
use limeshot_engine::{BitDepth, ExportFormat, ExportRange, RenderSettings};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  limeshot-cli info <project.lime>\n  limeshot-cli render <project.lime> <out.wav|out.mp3> [--pattern] [--rate N] [--bits 16|24|32f] [--bitrate N] [--dither] [--split]\n  limeshot-cli run <commands.json> [--project in.lime] [--save out.lime] [--render out.wav]\n  limeshot-cli demo <out.lime> [--render out.wav]\n  limeshot-cli play <project.lime> [--pattern] [--seconds N]\n  limeshot-cli midi <project.lime> <out.mid> [--pattern]\n  limeshot-cli import-midi <in.mid> <out.lime>\n  limeshot-cli import-flp <in.flp> [out.lime]"
    );
    eprintln!("  --allow-external-samples: explicitly allow embedding audio outside the loaded project folder");
    ExitCode::from(2)
}

struct Opts {
    positional: Vec<String>,
    pattern: bool,
    allow_external_samples: bool,
    rate: u32,
    depth: BitDepth,
    bitrate_kbps: u32,
    dither: bool,
    split: bool,
    project: Option<PathBuf>,
    save: Option<PathBuf>,
    render: Option<PathBuf>,
    seconds: f32,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        positional: Vec::new(),
        pattern: false,
        allow_external_samples: false,
        rate: 44_100,
        depth: BitDepth::Int24,
        bitrate_kbps: 320,
        dither: false,
        split: false,
        project: None,
        save: None,
        render: None,
        seconds: 5.0,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--pattern" => o.pattern = true,
            "--dither" => o.dither = true,
            "--split" => o.split = true,
            "--allow-external-samples" => o.allow_external_samples = true,
            "--rate" => o.rate = val()?.parse().map_err(|_| "bad --rate".to_string())?,
            "--bitrate" => o.bitrate_kbps = val()?.parse().map_err(|_| "bad --bitrate".to_string())?,
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
            "--seconds" => o.seconds = val()?.parse().map_err(|_| "bad --seconds".to_string())?,
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            s => o.positional.push(s.to_string()),
        }
    }
    Ok(o)
}

/// Load a project with absolute sample paths (as the app keeps them in memory).
fn load(path: &Path, allow_external_samples: bool) -> Result<Project, String> {
    let mut p = limeshot_project::load(path).map_err(|e| e.to_string())?;
    let dir = path.parent();
    for c in &mut p.channels {
        if let limeshot_core::Instrument::Sampler(s) = &mut c.instrument
            && let Some(r) = &mut s.sample
        {
            r.path = limeshot_project::resolve_sample_path(&r.path, dir)
                .to_string_lossy()
                .into_owned();
        }
    }
    if allow_external_samples {
        for channel in &mut p.channels {
            channel.extra.remove("lime-external-source");
        }
    } else {
        limeshot_ui::lime::restrict_loaded_sources(&mut p, dir);
    }
    Ok(p)
}

/// Normalize CLI playback sources and report conversion warnings before saving or rendering.
fn convert(project: &mut Project) {
    for warning in limeshot_core::lime::convert_project(project, &limeshot_ui::lime::embedded_audio) {
        eprintln!("warning: {warning}");
    }
}

/// Save a normalized copy without changing the caller project.
fn save_converted(project: &Project, path: &Path) -> Result<(), String> {
    let mut project = project.clone();
    convert(&mut project);
    limeshot_project::save(&project, path).map_err(|e| e.to_string())
}

fn render(project: &Project, out: &Path, o: &Opts) -> Result<(), String> {
    let format = match out
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("wav") => ExportFormat::Wav(o.depth),
        Some("mp3") => ExportFormat::Mp3 {
            bitrate_kbps: o.bitrate_kbps,
        },
        _ => return Err("audio output must have a .wav or .mp3 extension".into()),
    };
    format.validate(o.rate).map_err(|error| error.to_string())?;
    // Instruments and effects are the Lime VST3s (D39), as in the app.
    let mut converted = project.clone();
    convert(&mut converted);
    let project = &converted;
    let mut cache = SampleCache::new();
    for r in project.sample_refs() {
        if let Err(e) = cache.load_blocking(&r.path) {
            eprintln!("warning: {e}");
        }
    }
    let pattern = project.patterns.first().map(|p| p.id).ok_or("project has no pattern")?;
    let snap = Arc::new(limeshot_sequencer::compile(project, pattern, &cache));
    let range = if o.pattern {
        ExportRange::Pattern
    } else {
        ExportRange::Song
    };
    let settings = RenderSettings {
        sample_rate: o.rate,
        range,
        dither: o.dither,
        ..RenderSettings::default()
    };
    // Plugin channels and inserts get instances of their own, restored from the saved state.
    let mut plugins = limeshot_ui::plugins::PluginHost::new(None);
    let (processors, instances) = plugins.offline(project, o.rate)?;
    let (rep, stems) = if o.split {
        let names: Vec<String> = project.mixer.tracks.iter().map(|t| t.name.clone()).collect();
        let r = limeshot_engine::export_stems_with(snap, &settings, format, out, &names, processors)
            .map_err(|e| e.to_string())?;
        (r.master, r.stems)
    } else {
        let r =
            limeshot_engine::export_audio_with(snap, &settings, format, out, processors).map_err(|e| e.to_string())?;
        (r, Vec::new())
    };
    drop(instances);
    plugins.collect_returned();
    println!(
        "rendered {} — {:.2} s ({} frames at {} Hz), peak {:.1} dBFS",
        out.display(),
        rep.seconds(),
        rep.frames,
        rep.sample_rate,
        limeshot_dsp::gain_to_db(rep.peak)
    );
    for stem in stems {
        println!("  mixer track: {}", stem.display());
    }
    Ok(())
}

/// Play through the system audio device and report what the realtime callback produced.
fn play(project: &Project, o: &Opts) -> Result<(), String> {
    use limeshot_audio::{AudioConfig, Msg, Player};
    let mut converted = project.clone();
    convert(&mut converted);
    let project = &converted;
    let mut cache = SampleCache::new();
    for r in project.sample_refs() {
        if let Err(e) = cache.load_blocking(&r.path) {
            eprintln!("warning: {e}");
        }
    }
    let pattern = project.patterns.first().map(|p| p.id).ok_or("project has no pattern")?;
    let snap = Arc::new(limeshot_sequencer::compile(project, pattern, &cache));
    let mut player = Player::start(&AudioConfig::default(), snap.clone());
    // Effect processors and plugins are built here (control thread) and sent before re-sending
    // the snapshot.

    let mut plugins = limeshot_ui::plugins::PluginHost::new(None);
    let sr = player.sample_rate();
    let errors = plugins.sync(project, 0, sr, &mut |m| {
        player.send(m);
    });
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    player.send(Msg::Snapshot(snap));
    match player.output_description() {
        Ok(name) => println!("output: {name} at {} Hz", player.sample_rate()),
        Err(e) => println!("no output device ({e}); running the silent clock"),
    }
    let mode = if o.pattern {
        limeshot_engine::PlayMode::Pattern
    } else {
        limeshot_engine::PlayMode::Song
    };
    player.send(Msg::Mode(mode));
    player.send(Msg::Play);
    let start = std::time::Instant::now();
    let (mut peak, mut max_load, mut max_voices, mut max_pos) = (0.0f32, 0.0f32, 0, 0);
    while start.elapsed().as_secs_f32() < o.seconds {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let s = player.status();
        let (l, r) = s.peak(0);
        peak = peak.max(l).max(r);
        max_load = max_load.max(s.load());
        max_voices = max_voices.max(s.voices());
        max_pos = max_pos.max(s.position());
        player.collect_garbage();
    }
    let callbacks = player.status().callbacks();
    println!(
        "played {:.1} s: {} callbacks, master peak {:.1} dBFS, max {} voices, max load {:.1}%, furthest position {}",
        o.seconds,
        callbacks,
        limeshot_dsp::gain_to_db(peak),
        max_voices,
        max_load * 100.0,
        limeshot_core::BarBeatTick::from_ticks(max_pos, project.time_signature)
    );
    player.send(Msg::Stop);
    std::thread::sleep(std::time::Duration::from_millis(100));
    if callbacks == 0 || peak <= 0.0 {
        return Err("no audio was produced".into());
    }
    Ok(())
}

fn info(p: &Project) {
    let sig = p.time_signature;
    println!("title:      {}", p.title);
    println!("tempo:      {} BPM, {}", p.tempo, sig);
    println!("channels:   {}", p.channels.len());
    for c in &p.channels {
        println!(
            "  - {} ({}) → mixer {}",
            c.name,
            c.instrument.kind_label(),
            c.mixer_track
        );
    }
    println!("patterns:   {}", p.patterns.len());
    for pat in &p.patterns {
        println!(
            "  - {}: {} notes, {} bars",
            pat.name,
            pat.notes.len(),
            pat.length(sig) / sig.ticks_per_bar()
        );
    }
    println!(
        "clips:      {} (song {} bars)",
        p.playlist.clips.len(),
        p.playlist.song_length(sig) / sig.ticks_per_bar()
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((cmd, rest)) = args.split_first() else {
        return usage();
    };
    let o = match parse(rest) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return usage();
        }
    };
    let result = match cmd.as_str() {
        "info" => match o.positional.first() {
            Some(p) => load(Path::new(p), o.allow_external_samples).map(|p| info(&p)),
            None => return usage(),
        },
        "render" => match (o.positional.first(), o.positional.get(1)) {
            (Some(p), Some(out)) => {
                load(Path::new(p), o.allow_external_samples).and_then(|proj| render(&proj, Path::new(out), &o))
            }
            _ => return usage(),
        },
        "play" => match o.positional.first() {
            Some(p) => load(Path::new(p), o.allow_external_samples).and_then(|proj| play(&proj, &o)),
            None => return usage(),
        },
        "run" => {
            let Some(script) = o.positional.first() else {
                return usage();
            };
            (|| {
                let text = std::fs::read_to_string(script).map_err(|e| format!("{script}: {e}"))?;
                let cmds: Vec<Command> = serde_json::from_str(&text).map_err(|e| format!("{script}: {e}"))?;
                let mut doc = Document::new(match &o.project {
                    Some(p) => load(p, o.allow_external_samples)?,
                    None => Project::new(),
                });
                for (i, c) in cmds.into_iter().enumerate() {
                    let label = c.label();
                    let out = doc
                        .execute(c)
                        .map_err(|e| format!("command {} ({label}): {e}", i + 1))?;
                    if let Some(id) = out.0 {
                        println!("{}: {label} → id {id}", i + 1);
                    }
                }
                if let Some(s) = &o.save {
                    save_converted(doc.project(), s)?;
                    println!("saved {}", s.display());
                }
                if let Some(r) = &o.render {
                    render(doc.project(), r, &o)?;
                }
                Ok(())
            })()
        }
        "midi" => match (o.positional.first(), o.positional.get(1)) {
            (Some(p), Some(out)) => load(Path::new(p), o.allow_external_samples).and_then(|proj| {
                // The song, or with --pattern the first pattern (as `render` does).
                let pattern = proj.patterns.first().map(|p| p.id).ok_or("project has no pattern")?;
                let snap = limeshot_sequencer::compile(&proj, pattern, &limeshot_sequencer::NoSamples);
                let program = if o.pattern { &snap.pattern } else { &snap.song };
                let file = limeshot_project::midi::from_program(&proj, program);
                if file.tracks.is_empty() {
                    return Err("nothing to export: no notes".into());
                }
                std::fs::write(out, limeshot_project::midi::write(&file)).map_err(|e| format!("{out}: {e}"))?;
                let notes: usize = file.tracks.iter().map(|t| t.notes.len()).sum();
                println!("wrote {out}: {} tracks, {notes} notes", file.tracks.len());
                Ok(())
            }),
            _ => return usage(),
        },
        "import-flp" => match o.positional.first() {
            Some(input) => (|| {
                let factory = limeshot_project::flp::default_factory_dir();
                let path = Path::new(input);
                let (mut p, r) =
                    limeshot_project::flp::import_file(path, factory.as_deref()).map_err(|e| e.to_string())?;
                if !o.allow_external_samples {
                    limeshot_ui::lime::restrict_loaded_sources(&mut p, path.parent());
                }
                println!(
                    "{input}: FL Studio {} — {}",
                    r.version.as_deref().unwrap_or("?"),
                    r.summary()
                );
                if let Some(out) = o.positional.get(1) {
                    save_converted(&p, Path::new(out))?;
                    println!("saved {out}");
                }
                Ok(())
            })(),
            None => return usage(),
        },
        "import-midi" => match (o.positional.first(), o.positional.get(1)) {
            (Some(input), Some(out)) => (|| {
                let path = Path::new(input);
                let file = limeshot_project::midi::read_file(path).map_err(|e| e.to_string())?;
                let name = path
                    .file_stem()
                    .map_or("MIDI".into(), |s| s.to_string_lossy().into_owned());
                let mut doc = Document::new(Project::new());
                let cmd = limeshot_project::midi::import_command(&file, &name, true).map_err(|e| e.to_string())?;
                doc.execute(cmd).map_err(|e| e.to_string())?;
                save_converted(doc.project(), Path::new(out))?;
                println!("saved {out}: {} channels", doc.project().channels.len());
                Ok(())
            })(),
            _ => return usage(),
        },
        "demo" => {
            let Some(out) = o.positional.first() else {
                return usage();
            };
            (|| {
                let samples = limeshot_app::bundled_samples_dir().ok_or("bundled samples not found")?;
                let doc = limeshot_app::demo::build(&samples)?;
                save_converted(doc.project(), Path::new(out))?;
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
