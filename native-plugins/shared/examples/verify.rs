//! Load the actual exported binaries through the VST3 ABI, rather than calling their DSP directly.
use limeshot_plugins::{NoteEvent, NoteEventKind, Transport, Vst3Instance, Vst3Module};
use std::{error::Error, path::PathBuf};

fn require(ok: bool, message: impl Into<String>) -> Result<(), Box<dyn Error>> {
    if ok {
        Ok(())
    } else {
        Err(std::io::Error::other(message.into()).into())
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let folder = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| "dist".into());
    for (index, name) in lime_native::parameters::NAMES.iter().enumerate() {
        let path = folder.join(format!("{name}.vst3"));
        let module = Vst3Module::load(&path)?;
        require(
            module.classes().len() == 1,
            format!("{name}: expected one plugin class"),
        )?;
        let class = &module.classes()[0];
        require(class.name == *name, format!("{name}: wrong plugin name {}", class.name))?;
        for rate in [44100.0, 48000.0, 96000.0] {
            for size in [1, 64, 333, 1024] {
                let (mut instance, mut processor) = Vst3Instance::new(&module, &class.class_id, rate, 1024)?;
                require(instance.create_editor().is_some(), format!("{name}: no native editor"))?;
                let mut buf = vec![0.0_f32; size * 2];
                let on = [NoteEvent {
                    offset: if size > 64 { 37 } else { 0 },
                    kind: NoteEventKind::On {
                        key: 60,
                        velocity: 0.8,
                        id: 1,
                    },
                }];
                let mut peak = 0.0_f32;
                for block in 0..30 {
                    for (i, s) in buf.iter_mut().enumerate() {
                        *s = if index < 2 {
                            0.0
                        } else {
                            ((i + block * size * 2) as f32 * 0.13).sin() * 0.8
                        };
                    }
                    processor.process(
                        &mut buf,
                        if block == 0 && index < 2 { &on } else { &[] },
                        &Transport::default(),
                    );
                    require(buf.iter().all(|v| v.is_finite()), format!("{name}: invalid audio"))?;
                    if block == 0 && index == 0 && size > 64 {
                        require(
                            buf[..74].iter().all(|s| *s == 0.0),
                            "synth note began before its timestamp",
                        )?;
                    }
                    peak = buf.iter().fold(peak, |a, v| a.max(v.abs()));
                }
                require(index == 1 || peak > 0.00001, format!("{name}: unexpectedly silent"))?;
                let param = instance
                    .params()
                    .iter()
                    .find(|p| p.automatable() && !p.is_bypass() && !p.read_only())
                    .map(|p| p.id)
                    .ok_or_else(|| std::io::Error::other("no automatable parameters"))?;
                instance.set_param(param, 0.37);
                processor.process(&mut buf, &[], &Transport::default());
                let value = instance.param(param);
                let state = instance.state()?;
                let (mut recalled, mut recalled_processor) = Vst3Instance::new(&module, &class.class_id, rate, 1024)?;
                recalled.set_state(&state)?;
                require(
                    (recalled.param(param) - value).abs() < 0.00001,
                    format!("{name}: parameter state was not recalled"),
                )?;
                recalled_processor.process(&mut buf, &[], &Transport::default());
                require(buf.iter().all(|v| v.is_finite()), format!("{name}: bad recalled audio"))?;
                // Exercise host-scheduled automation at an offset inside the block.
                processor.set_param_at(param, if size > 64 { 37 } else { 0 }, 0.8);
                processor.process(&mut buf, &[], &Transport::default());
                require(
                    buf.iter().all(|v| v.is_finite()),
                    format!("{name}: bad automated audio"),
                )?;
            }
        }
        println!(
            "PASS {name}: VST3 load, native editor creation, audio/MIDI offsets, automation, state; 3 sample rates × 4 buffer sizes"
        );
    }
    Ok(())
}
