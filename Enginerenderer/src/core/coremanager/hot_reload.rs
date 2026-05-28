use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct WatchedAsset {
    pub path: PathBuf,
    pub last_modified: SystemTime,
}

impl WatchedAsset {
    pub fn new(path: impl Into<PathBuf>) -> Option<Self> {
        let path = path.into();
        let last_modified = std::fs::metadata(&path).ok()?.modified().ok()?;
        Some(Self { path, last_modified })
    }

    pub fn has_changed(&self) -> bool {
        std::fs::metadata(&self.path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| t > self.last_modified)
            .unwrap_or(false)
    }

    pub fn update_timestamp(&mut self) {
        if let Some(t) = std::fs::metadata(&self.path).ok().and_then(|m| m.modified().ok()) {
            self.last_modified = t;
        }
    }
}

pub struct HotReloader {
    watched: Vec<WatchedAsset>,
}

impl HotReloader {
    pub fn new() -> Self {
        Self { watched: Vec::new() }
    }

    pub fn watch(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        if let Some(asset) = WatchedAsset::new(&path) {
            self.watched.push(asset);
        }
    }

    pub fn poll(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        for asset in &mut self.watched {
            if asset.has_changed() {
                changed.push(asset.path.clone());
                asset.update_timestamp();
            }
        }
        changed
    }

    pub fn watch_count(&self) -> usize {
        self.watched.len()
    }

    pub fn watched_paths(&self) -> impl Iterator<Item = &Path> {
        self.watched.iter().map(|a| a.path.as_path())
    }
}

impl std::fmt::Debug for HotReloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HotReloader")
            .field("watch_count", &self.watched.len())
            .finish()
    }
}
