fn main() {
    let folder = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    if let Some(folder) = &folder {
        std::fs::create_dir_all(folder).unwrap();
    }
    for kind in ["tone", "impulses", "percussion", "polyphonic"] {
        let rate = 48000;
        let n = rate as usize;
        let x: Vec<f32> = (0..n)
            .flat_map(|i| {
                let t = i as f32 / rate as f32;
                let a = match kind {
                    "tone" => (t * 440. * std::f32::consts::TAU).sin() * 0.5,
                    "impulses" => {
                        if i % 12000 == 0 {
                            1.
                        } else {
                            0.
                        }
                    }
                    "percussion" => (t * 150. * std::f32::consts::TAU).sin() * (-((i % 12000) as f32) / 500.).exp(),
                    _ => [220., 277.18, 329.63]
                        .iter()
                        .map(|f| (t * f * std::f32::consts::TAU).sin() / 6.)
                        .sum(),
                };
                [a, -a]
            })
            .collect();
        for ratio in [0.5, 1., 2.] {
            for candidate in ["wsola", "timestretch", "Lime WSOLA"] {
                let now = std::time::Instant::now();
                let y = if candidate == "Lime WSOLA" {
                    limeshot_dsp::stretch::prepare(
                        &limeshot_dsp::SampleData::from_stereo(rate, x.clone()),
                        ratio,
                        0.,
                        true,
                        &std::sync::atomic::AtomicBool::new(false),
                    )
                    .unwrap()
                    .data
                } else if candidate == "wsola" {
                    wsola::stretch(&x, rate, 2, (1. / ratio) as f32).unwrap()
                } else {
                    timestretch::stretch(
                        &x,
                        &timestretch::StretchParams::new(ratio)
                            .with_sample_rate(rate)
                            .with_channels(2),
                    )
                    .unwrap()
                };
                if let Some(folder) = &folder {
                    let name = format!("{kind}-{ratio}-{candidate}.wav");
                    limeshot_dsp::write_wav(&folder.join(name), rate, &y, limeshot_dsp::BitDepth::Float32).unwrap();
                }
                let rms = (y.iter().map(|v| v * v).sum::<f32>() / y.len() as f32).sqrt();
                let stereo = y.chunks_exact(2).map(|v| (v[0] + v[1]).abs()).fold(0f32, f32::max);
                println!(
                    "{kind},{ratio},{candidate},frames={},rms={rms:.5},stereo_error={stereo:.7},ms={}",
                    y.len() / 2,
                    now.elapsed().as_millis()
                );
            }
        }
    }
}
