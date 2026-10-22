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
