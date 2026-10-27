//! Worker-only preparation of Lime Sampler's portable sample state. Generic plugins never use it.
use limeshot_core::plugin::{base64_decode, base64_encode};
use limeshot_dsp::{SampleData, stretch};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};

/// Retain original embedded PCM in `preparation-v1`; `pcm` always holds the playable result.
/// The combined JSON must fit the host state limit. Disk sources must be explicitly converted
/// to embedded mode first, so preparation cannot implicitly read an unapproved external path.
pub fn prepare_sample(
    text: &str,
    ratio: f64,
    pitch: f64,
    stretch_mode: bool,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let mut sample: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if sample["keep_on_disk"].as_bool() == Some(true) {
        return Err("Switch Keep on disk off before preparing (3 million frame limit)".into());
    }
    let original = sample
        .get("preparation-v1")
        .and_then(|p| p.get("original"))
        .cloned()
        .unwrap_or_else(|| sample.clone());
    if !original.is_object()
        || original.get("preparation-v1").is_some()
        || original["keep_on_disk"].as_bool() == Some(true)
    {
        return Err("Invalid original embedded source".into());
    }
    let pcm = original["pcm"].as_str().ok_or("No embedded source")?;
    let limit = (stretch::MAX_PREPARED_FRAMES * 8).div_ceil(3) * 4;
    if pcm.len() > limit {
        return Err("Original exceeds 3 million frames".into());
    }
    let rate = original["sample_rate"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("Invalid source rate")?;
    let bytes = base64_decode(pcm).ok_or("Invalid PCM")?;
    if bytes.len() % 8 != 0 {
        return Err("Incomplete stereo frame".into());
    }
    let source = SampleData::from_stereo(
        rate,
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect(),
    );
    let prepared = stretch::prepare(&source, ratio, pitch, stretch_mode, cancel).map_err(|e| e.to_string())?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Preparation cancelled".into());
    }
    let bytes: Vec<u8> = prepared.data.iter().flat_map(|v| v.to_le_bytes()).collect();
    sample["pcm"] = json!(base64_encode(&bytes));
    sample["sample_rate"] = json!(rate);
    sample["keep_on_disk"] = json!(false);
    sample["preparation-v1"] =
        json!({"original":original,"ratio":ratio,"pitch":pitch,"mode":if stretch_mode {"Stretch"}else{"Resample"}});
    let result = sample.to_string();
    // Leave room for escaping, component controls, and the host envelope.
    if result.len() > limeshot_core::plugin::MAX_STATE_BYTES - 1_048_576 {
        return Err("Original plus prepared audio exceeds the portable state limit".into());
    }
    Ok(result)
}

pub fn restore_sample(text: &str) -> Result<String, String> {
    let sample: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let original = sample
        .get("preparation-v1")
        .and_then(|p| p.get("original"))
        .unwrap_or(&sample);
    if !original.is_object()
        || original.get("preparation-v1").is_some()
        || original["keep_on_disk"].as_bool() == Some(true)
    {
        return Err("Restore requires the retained embedded original".into());
    }
    let rate = original["sample_rate"].as_u64().ok_or("Invalid original rate")?;
    let pcm = original["pcm"].as_str().ok_or("No original PCM")?;
    let max = stretch::MAX_PREPARED_FRAMES;
    if !(8000..=384000).contains(&rate) || pcm.len() > (max * 8).div_ceil(3) * 4 {
        return Err("Invalid original source bounds".into());
    }
    let bytes = base64_decode(pcm).ok_or("Invalid original PCM")?;
    if bytes.len() < 16
        || bytes.len() % 8 != 0
        || bytes.len() > max * 8
        || bytes
            .as_chunks::<4>()
            .0
            .iter()
            .any(|b| !f32::from_le_bytes(*b).is_finite())
    {
        return Err("Invalid original stereo PCM".into());
    }
    Ok(original.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recipes_reprepare_from_original_restore_and_reject_unapproved_disk() {
        let pcm: Vec<u8> = (0..8000)
            .flat_map(|i| [(i as f32 * 0.2).sin(); 2])
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let source =
            json!({"name":"fixture","source":"fixture.wav","sample_rate":8000,"pcm":base64_encode(&pcm)}).to_string();
        let cancel = AtomicBool::new(false);
        assert!(restore_sample(r#"{"preparation-v1":{"original":{"pcm":"AA==","sample_rate":0}}}"#).is_err());
        let stretched = prepare_sample(&source, 2., 0., true, &cancel).unwrap();
        let again = prepare_sample(&stretched, 0.5, 12., true, &cancel).unwrap();
        let direct = prepare_sample(&source, 0.5, 12., true, &cancel).unwrap();
        assert_eq!(again, direct);
        let restored: Value = serde_json::from_str(&restore_sample(&again).unwrap()).unwrap();
        assert_eq!(restored, serde_json::from_str::<Value>(&source).unwrap());
        assert!(
            prepare_sample(
                r#"{"keep_on_disk":true,"source":"/private.wav"}"#,
                1.,
                0.,
                true,
                &cancel
            )
            .is_err()
        );
        assert!(prepare_sample(r#"{"pcm":"AA==","sample_rate":48000}"#, 1., 0., true, &cancel).is_err());
    }
}
