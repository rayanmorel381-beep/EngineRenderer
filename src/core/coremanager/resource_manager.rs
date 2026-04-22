
use std::path::PathBuf;

/// Tracks generated output files during runtime.
#[derive(Debug, Clone, Default)]
pub struct ResourceTracker {
    frame_outputs: Vec<PathBuf>,
}

impl ResourceTracker {
    /// Creates an empty resource tracker.
    pub fn new() -> Self {
        Self { frame_outputs: Vec::new() }
    }

    /// Registers a new output file path.
    pub fn record_output(&mut self, path: PathBuf) {
        self.frame_outputs.push(path);
    }

    /// Returns the number of tracked outputs.
    pub fn output_count(&self) -> usize {
        self.frame_outputs.len()
    }

    /// Returns all tracked output paths.
    pub fn outputs(&self) -> &[PathBuf] {
        &self.frame_outputs
    }

    /// Returns true when at least one output was recorded.
    pub fn has_outputs(&self) -> bool {
        !self.frame_outputs.is_empty()
    }
}
