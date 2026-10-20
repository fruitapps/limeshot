//! Sample cache with a background loader thread.
//!
//! The UI asks for samples by their stored project path; the cache resolves the path against the
//! project folder and decodes it on a worker thread. Decoded samples stay cached for the life of
//! the cache, which also guarantees the audio thread never drops the last reference to a buffer.

use limeshot_dsp::{SampleData, load_wav};
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
    jobs: Option<Sender<PathBuf>>,
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
        let (job_tx, job_rx) = channel::<PathBuf>();
        let (res_tx, res_rx) = channel::<LoadResult>();
        let spawned = std::thread::Builder::new()
            .name("limeshot-sample-loader".into())
            .spawn(move || {
                while let Ok(path) = job_rx.recv() {
                    let r = load_wav(&path).map(Arc::new).map_err(|e| e.to_string());
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
            jobs,
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
        match &self.jobs {
            Some(tx) if tx.send(path.clone()).is_ok() => {
                self.entries.insert(path, SampleStatus::Loading);
            }
            _ => {
                // No loader thread: decode here.
                let r = load_wav(&path).map(Arc::new).map_err(|e| e.to_string());
                self.entries.insert(path, status(r));
            }
        }
    }

    /// Decode now, on this thread (CLI, tests).
    pub fn load_blocking(&mut self, stored: &str) -> Result<Arc<SampleData>, String> {
        let path = self.resolve(stored);
        if let Some(SampleStatus::Ready(s)) = self.entries.get(&path) {
            return Ok(s.clone());
        }
        let r = load_wav(&path).map(Arc::new).map_err(|e| e.to_string());
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

    pub fn status(&self, stored: &str) -> Option<&SampleStatus> {
        self.entries.get(&self.resolve(stored))
    }

    pub fn status_of_path(&self, path: &Path) -> Option<&SampleStatus> {
        self.entries.get(path)
    }

    /// Forget a failed load so it can be retried (e.g. after the file was restored).
    pub fn retry(&mut self, stored: &str) {
        let path = self.resolve(stored);
        if matches!(self.entries.get(&path), Some(SampleStatus::Failed(_))) {
            self.entries.remove(&path);
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
}
