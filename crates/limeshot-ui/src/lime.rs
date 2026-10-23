//! LimeShot's instruments and effects are the Lime Native VST3 plugins (decision D39). The app
//! installs [`normalizer`] on its document, so former built-in instruments and effects, whether
//! opened from a project or created by an edit, become Lime plugins before they reach the engine.

use std::path::Path;

use limeshot_core::{EmbeddedAudio, Project, lime};

/// Decode a sampler's sample (an absolute path in memory) for Lime Sampler's state.
pub fn embedded_audio(stored: &str) -> Option<EmbeddedAudio> {
    let path = limeshot_project::resolve_sample_path(stored, None);
    let data = limeshot_dsp::load_audio(&path).ok()?;
    Some(EmbeddedAudio {
        name: Path::new(stored)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        sample_rate: data.sample_rate,
        stereo: data.data,
    })
}

/// The document normalizer that converts to Lime plugins.
pub fn normalizer() -> limeshot_core::document::Normalizer {
    Box::new(|project: &mut Project| {
        if lime::needs_conversion(project) {
            lime::convert_project(project, &embedded_audio)
        } else {
            Vec::new()
        }
    })
}

/// Decode only on document changes, never from drawing or the audio callback. A content key
/// shares cached PCM across parameter edits and channels, even after the source file goes away.
pub fn cache_embedded_sample(
    state: &limeshot_core::PluginState,
    cache: &mut limeshot_audio::assets::SampleCache,
) -> Option<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    let component: serde_json::Value = serde_json::from_slice(lime::component_state(state)?).ok()?;
    let text = component["fields"]["sample-v1"].as_str()?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hash);
    let path = std::path::PathBuf::from(format!("/.limeshot-embedded/{:016x}.wav", hash.finish()));
    if cache.status_of_path(&path).is_some() {
        return Some(path);
    }
    let sample: serde_json::Value = serde_json::from_str(text).ok()?;
    let rate = u32::try_from(sample["sample_rate"].as_u64()?).ok()?;
    let pcm = sample["pcm"].as_str()?;
    if !(8000..=384000).contains(&rate) || pcm.len() > (4_000_000usize * 8).div_ceil(3) * 4 {
        return None;
    }
    let bytes = limeshot_core::plugin::base64_decode(pcm)?;
    if bytes.len() < 16 || bytes.len() % 8 != 0 {
        return None;
    }
    let data = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| {
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            if v.is_finite() { v } else { 0.0 }
        })
        .collect();
    cache.insert_embedded(
        path.clone(),
        std::sync::Arc::new(limeshot_dsp::SampleData::from_stereo(rate, data)),
    );
    Some(path)
}
