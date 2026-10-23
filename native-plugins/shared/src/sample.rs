//! Samples are decoded when state is loaded/activated, never during process().
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossbeam_queue::ArrayQueue;
use limeshot_dsp::SampleData;
use nice_plug::params::persist::PersistentField;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

pub const MAX_FRAMES: usize = limeshot_core::lime::MAX_EMBEDDED_FRAMES;
const MAX_PCM_TEXT: usize = (MAX_FRAMES * 8).div_ceil(3) * 4;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EmbeddedSample {
    pub name: String,
    pub sample_rate: u32,
    pub pcm: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SampleError {
    #[error("Sample exceeds the 4 million frame limit")]
    TooLong,
    #[error("Invalid sample data")]
    Invalid,
    #[error("{0}")]
    Decode(#[from] limeshot_dsp::SampleError),
    #[error("{0}")]
    Wav(#[from] hound::Error),
}

impl EmbeddedSample {
    /// Read a WAV into embedded stereo PCM, keeping its filename for display.
    /// Mono is duplicated; extra channels are discarded and non-finite values become zero.
    ///
    /// # Errors
    /// Returns `TooLong` above 4 million frames, `Wav` for errors opening or reading the
    /// WAV header, or `Decode` for errors propagated by the shared WAV loader.
    pub fn load(path: &Path) -> Result<Self, SampleError> {
        let reader = hound::WavReader::open(path)?;
        if reader.duration() as usize > MAX_FRAMES {
            return Err(SampleError::TooLong);
        }
        let sample = limeshot_dsp::load_wav(path)?;
        let bytes: Vec<u8> = sample.data.iter().flat_map(|f| f.to_le_bytes()).collect();
        Ok(Self {
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
            sample_rate: sample.sample_rate,
            pcm: STANDARD.encode(bytes),
        })
    }

    /// Decode embedded stereo PCM, replacing non-finite values with zero.
    /// Empty PCM returns `Ok(None)` without validating the sample rate.
    ///
    /// # Errors
    /// Returns `TooLong` if the encoded text exceeds the frame-limit allowance, or `Invalid`
    /// for a rate outside 8,000–384,000 Hz, malformed base64, incomplete stereo frames,
    /// or fewer than two frames.
    fn decode(&self) -> Result<Option<Arc<SampleData>>, SampleError> {
        if self.pcm.is_empty() {
            return Ok(None);
        }
        if self.pcm.len() > MAX_PCM_TEXT {
            return Err(SampleError::TooLong);
        }
        if !(8000..=384000).contains(&self.sample_rate) {
            return Err(SampleError::Invalid);
        }
        let bytes = STANDARD.decode(&self.pcm).map_err(|_| SampleError::Invalid)?;
        if bytes.len() % 8 != 0 || bytes.len() < 16 {
            return Err(SampleError::Invalid);
        }
        let mut data = Vec::with_capacity(bytes.len() / 4);
        for b in bytes.as_chunks::<4>().0 {
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            data.push(if v.is_finite() { v } else { 0.0 });
        }
        Ok(Some(Arc::new(SampleData::from_stereo(self.sample_rate, data))))
    }
}

/// Persistent state is control-thread-only. Bounded queues transfer decoded samples to the
/// processor and return retired samples for destruction on the next control-thread state update.
#[derive(Clone)]
pub struct SampleStore {
    state: Arc<RwLock<(EmbeddedSample, Option<Arc<SampleData>>)>>,
    pending: Arc<ArrayQueue<Option<Arc<SampleData>>>>,
    retired: Arc<ArrayQueue<Arc<SampleData>>>,
}
impl Default for SampleStore {
    /// Preallocate bounded handoff queues before the processor can enter its realtime callback.
    fn default() -> Self {
        Self {
            state: Arc::new(RwLock::new(Default::default())),
            pending: Arc::new(ArrayQueue::new(8)),
            retired: Arc::new(ArrayQueue::new(8)),
        }
    }
}
impl SampleStore {
    /// Share decoded audio during activation, outside audio processing.
    pub fn audio(&self) -> Option<Arc<SampleData>> {
        self.state.read().1.clone()
    }
    /// Nonblocking audio-thread handoff. Empty samples are explicit updates.
    pub(crate) fn take_audio(&self) -> Option<Option<Arc<SampleData>>> {
        self.pending.pop()
    }
    /// Whether adopting more audio must wait for control-thread sample collection.
    pub(crate) fn retirement_full(&self) -> bool {
        self.retired.is_full()
    }
    /// Keep the old audio alive until a control-thread state update collects it.
    pub(crate) fn retire_audio(&self, audio: Arc<SampleData>) -> Result<(), Arc<SampleData>> {
        self.retired.push(audio)
    }
    /// Copy the stored display name under a control-thread read lock.
    pub fn name(&self) -> String {
        self.state.read().0.name.clone()
    }
}

impl<'a> PersistentField<'a, EmbeddedSample> for SampleStore {
    /// Decode and replace the embedded sample and cached audio under a write lock.
    /// Decode errors clear both to their defaults rather than reaching the caller.
    fn set(&self, sample: EmbeddedSample) {
        while self.retired.pop().is_some() {}
        let mut state = self.state.write();
        if state.0.name == sample.name && state.0.sample_rate == sample.sample_rate && state.0.pcm == sample.pcm {
            return;
        }
        let (sample, audio) = match sample.decode() {
            Ok(audio) => (sample, audio),
            Err(_) => Default::default(),
        };
        // Serialize producers with the persistence lock, which the processor never takes.
        *state = (sample, audio.clone());
        let mut update = audio;
        loop {
            match self.pending.push(update) {
                Ok(()) => break,
                Err(audio) => {
                    update = audio;
                    // Coalesce old, unconsumed updates on this control thread.
                    self.pending.pop();
                }
            }
        }
    }
    /// Return the callback's result while holding a read lock on the embedded sample.
    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&EmbeddedSample) -> R,
    {
        f(&self.state.read().0)
    }
}
