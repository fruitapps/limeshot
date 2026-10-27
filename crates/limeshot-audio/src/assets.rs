//! Sample cache with a background loader thread.
//!
//! The UI asks for samples by their stored project path; the cache resolves the path against the
//! project folder and decodes it on a worker thread. Decoded samples stay cached for the life of
//! the cache, which also guarantees the audio thread never drops the last reference to a buffer.

use limeshot_dsp::SampleData;

fn load_audio(path: &Path, auto_disk: bool) -> Result<SampleData, String> {
    let sample = match limeshot_dsp::load_audio_bounded(path, limeshot_core::lime::MAX_EMBEDDED_FRAMES) {
        Ok(sample) => sample,
        Err(limeshot_dsp::SampleError::TooLong { .. }) if auto_disk => {
            limeshot_dsp::disk::load_disk(path).map_err(|e| e.to_string())?
        }
        Err(error) => return Err(error.to_string()),
    };
    if sample.frames() < 2 || !(8_000..=384_000).contains(&sample.sample_rate) {
        return Err("Sample must have at least two frames at 8–384 kHz".into());
    }
    Ok(sample)
}
use limeshot_sequencer::SampleLookup;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Debug, Clone)]
pub enum SampleStatus {
    Loading,
    Ready(Arc<SampleData>),
    Failed(String),
}

type LoadResult = (PathBuf, Result<Arc<SampleData>, String>);

pub struct SampleCache {
    project_dir: Option<PathBuf>,
    entries: HashMap<PathBuf, SampleStatus>,
    explicit_disk: std::collections::HashSet<PathBuf>,
    jobs: Option<Sender<(PathBuf, bool)>>,
    pub auto_keep_long_audio_on_disk: bool,
    results: Receiver<LoadResult>,
}

impl Default for SampleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleCache {
    /// A cache with its own loader thread.
    pub fn new() -> Self {
        let (job_tx, job_rx) = channel::<(PathBuf, bool)>();
        let (res_tx, res_rx) = channel::<LoadResult>();
        let spawned = std::thread::Builder::new()
            .name("limeshot-sample-loader".into())
            .spawn(move || {
                while let Ok((path, auto_disk)) = job_rx.recv() {
                    let r = load_audio(&path, auto_disk).map(Arc::new).map_err(|e| e.to_string());
                    if res_tx.send((path, r)).is_err() {
                        break;
                    }
                }
            });
        let jobs = match spawned {
            Ok(_) => Some(job_tx),
            Err(e) => {
                log::error!("could not start the sample loader: {e}");
                None
            }
        };
        Self {
            project_dir: None,
            entries: HashMap::new(),
            explicit_disk: Default::default(),
            jobs,
            auto_keep_long_audio_on_disk: true,
            results: res_rx,
        }
    }

    /// Relative sample paths resolve against this folder (the open project's).
    pub fn set_project_dir(&mut self, dir: Option<PathBuf>) {
        self.project_dir = dir;
    }

    pub fn project_dir(&self) -> Option<&Path> {
        self.project_dir.as_deref()
    }

    pub fn resolve(&self, stored: &str) -> PathBuf {
        limeshot_project::resolve_sample_path(stored, self.project_dir.as_deref())
    }

    /// Start loading `stored` if it is not cached yet.
    pub fn request(&mut self, stored: &str) {
        let path = self.resolve(stored);
        self.request_path(path);
    }

    pub fn request_path(&mut self, path: PathBuf) {
        if self.entries.contains_key(&path) {
            return;
        }
        self.queue_path(path, self.auto_keep_long_audio_on_disk);
    }

    /// A persisted per-sampler Keep on disk choice overrides the automatic-import preference.
    pub fn request_disk_path(&mut self, path: PathBuf) {
        if matches!(self.entries.get(&path), Some(SampleStatus::Ready(_))) {
            return;
        }
        if !self.explicit_disk.insert(path.clone()) {
            return;
        }
        self.queue_path(path, true);
    }

    fn queue_path(&mut self, path: PathBuf, auto_disk: bool) {
        match &self.jobs {
            Some(tx) if tx.send((path.clone(), auto_disk)).is_ok() => {
                self.entries.insert(path, SampleStatus::Loading);
            }
            _ => {
                self.entries
                    .insert(path, SampleStatus::Failed("Sample loader is unavailable".into()));
            }
        }
    }

    /// Decode now, on this thread (CLI, tests).
    pub fn load_blocking(&mut self, stored: &str) -> Result<Arc<SampleData>, String> {
        let path = self.resolve(stored);
        if let Some(SampleStatus::Ready(s)) = self.entries.get(&path) {
            return Ok(s.clone());
        }
        let r = load_audio(&path, self.auto_keep_long_audio_on_disk)
            .map(Arc::new)
            .map_err(|e| e.to_string());
        self.entries.insert(path, status(r.clone()));
        r
    }

    /// Collect finished loads. Returns true if anything changed (rebuild the playback snapshot).
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok((path, r)) = self.results.try_recv() {
            if let Err(e) = &r {
                log::warn!("sample: {e}");
            }
            self.entries.insert(path, status(r));
            changed = true;
        }
        changed
    }

    /// Retain a decoded embedded source on the control thread, just like a file load.
    pub fn insert_embedded(&mut self, path: PathBuf, sample: Arc<SampleData>) -> bool {
        if self.entries.contains_key(&path) {
            return true;
        }
        let embedded = |key: &PathBuf| key.starts_with("/.limeshot-embedded");
        let usage = |entries: &HashMap<PathBuf, SampleStatus>| {
            entries
                .iter()
                .filter(|(key, _)| embedded(key))
                .fold((0, 0usize), |(count, bytes), (_, status)| {
                    (
                        count + 1,
                        bytes
                            + match status {
                                SampleStatus::Ready(s) => s.data.len() * 4,
                                _ => 0,
                            },
                    )
                })
        };
        let (count, bytes) = usage(&self.entries);
        if count >= 32 || bytes + sample.data.len() * 4 > 256 * 1024 * 1024 {
            // Snapshot/voice references keep active audio pinned. Only the control thread
            // releases an unused buffer, preserving the callback's no-large-free contract.
            self.entries.retain(|key, status| {
                !embedded(key) || !matches!(status,SampleStatus::Ready(audio) if Arc::strong_count(audio)==1)
            });
        }
        let (count, bytes) = usage(&self.entries);
        if count >= 32 || bytes + sample.data.len() * 4 > 256 * 1024 * 1024 {
            return false;
        }
        self.entries.insert(path, SampleStatus::Ready(sample));
        true
    }
    pub fn status(&self, stored: &str) -> Option<&SampleStatus> {
        self.entries.get(&self.resolve(stored))
    }

    /// How many samples are still being decoded.
    pub fn loading(&self) -> usize {
        self.entries
            .values()
            .filter(|s| matches!(s, SampleStatus::Loading))
            .count()
    }

    pub fn status_of_path(&self, path: &Path) -> Option<&SampleStatus> {
        self.entries.get(path)
    }

    /// Forget only a failed load, allowing a later request to decode the path again.
    pub fn remove_failed(&mut self, path: &Path) -> bool {
        if matches!(self.entries.get(path), Some(SampleStatus::Failed(_))) {
            self.entries.remove(path);
            self.explicit_disk.remove(path);
            true
        } else {
            false
        }
    }

    /// Forget a failed load so it can be retried (e.g. after the file was restored).
    pub fn retry(&mut self, stored: &str) {
        let path = self.resolve(stored);
        if self.remove_failed(&path) {
            self.request_path(path);
        }
    }
}

fn status(r: Result<Arc<SampleData>, String>) -> SampleStatus {
    match r {
        Ok(s) => SampleStatus::Ready(s),
        Err(e) => SampleStatus::Failed(e),
    }
}

impl SampleLookup for SampleCache {
    fn sample(&self, stored_path: &str) -> Option<Arc<SampleData>> {
        match self.status(stored_path) {
            Some(SampleStatus::Ready(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn loads_in_the_background_and_reports_failures() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.wav");
        limeshot_dsp::write_wav(&p, 48_000, &[0.1, 0.2, 0.3, 0.4], limeshot_dsp::BitDepth::Float32).unwrap();
        let mut c = SampleCache::new();
        c.set_project_dir(Some(dir.path().to_path_buf()));
        c.request("a.wav");
        c.request("missing.wav");
        let t0 = Instant::now();
        while (c.sample("a.wav").is_none() || matches!(c.status("missing.wav"), Some(SampleStatus::Loading)))
            && t0.elapsed() < Duration::from_secs(5)
        {
            c.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(c.sample("a.wav").unwrap().frames(), 2);
        assert!(matches!(c.status("missing.wav"), Some(SampleStatus::Failed(_))));
    }

    #[test]
    fn forgetting_failed_paths_preserves_other_states_and_allows_a_fresh_decode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("restored.wav");
        let mut c = SampleCache::new();
        assert!(!c.remove_failed(&path));
        c.request_path(path.clone());
        assert!(!c.remove_failed(&path));
        assert!(matches!(c.status_of_path(&path), Some(SampleStatus::Loading)));
        let t0 = Instant::now();
        while matches!(c.status_of_path(&path), Some(SampleStatus::Loading)) && t0.elapsed() < Duration::from_secs(5) {
            c.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(c.remove_failed(&path));
        assert!(c.status_of_path(&path).is_none());
        limeshot_dsp::write_wav(&path, 48_000, &[0.1; 4], limeshot_dsp::BitDepth::Float32).unwrap();
        c.request_path(path.clone());
        let t0 = Instant::now();
        while matches!(c.status_of_path(&path), Some(SampleStatus::Loading)) && t0.elapsed() < Duration::from_secs(5) {
            c.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        let Some(SampleStatus::Ready(sample)) = c.status_of_path(&path).cloned() else {
            panic!("restored sample was not decoded");
        };
        assert!(!c.remove_failed(&path));
        assert!(matches!(c.status_of_path(&path), Some(SampleStatus::Ready(s)) if Arc::ptr_eq(s, &sample)));
    }
    #[test]
    fn embedded_preview_budget_pins_active_audio_and_collects_unused_buffers() {
        let mut cache = SampleCache::new();
        let mut active = Vec::new();
        for i in 0..32 {
            let audio = Arc::new(SampleData::from_stereo(48000, vec![0.; 4]));
            assert!(cache.insert_embedded(PathBuf::from(format!("/.limeshot-embedded/{i}.wav")), audio.clone()));
            active.push(audio);
        }
        let next = PathBuf::from("/.limeshot-embedded/next.wav");
        let audio = Arc::new(SampleData::from_stereo(48000, vec![0.; 4]));
        assert!(!cache.insert_embedded(next.clone(), audio.clone()));
        assert_eq!(cache.entries.len(), 32);
        let old = Arc::downgrade(&active[0]);
        active.clear();
        assert!(cache.insert_embedded(next, audio));
        assert!(old.upgrade().is_none());
        assert_eq!(cache.entries.len(), 1);
    }
}
