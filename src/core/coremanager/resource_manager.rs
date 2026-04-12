//! Resource tracking for the engine runtime.
//!
//! [`ResourceTracker`] records frame outputs and resource state across
//! the render lifecycle.

use std::path::PathBuf;

/// Tracks resource outputs produced during rendering.
#[derive(Debug, Clone, Default)]
pub struct ResourceTracker {
    frame_outputs: Vec<PathBuf>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self { frame_outputs: Vec::new() }
    }

    /// Record a rendered frame output path.
    pub fn record_output(&mut self, path: PathBuf) {
        self.frame_outputs.push(path);
    }

    /// How many frame outputs have been recorded.
    pub fn output_count(&self) -> usize {
        self.frame_outputs.len()
    }

    /// All recorded output paths.
    pub fn outputs(&self) -> &[PathBuf] {
        &self.frame_outputs
    }

    /// Whether any frames have been written.
    pub fn has_outputs(&self) -> bool {
        !self.frame_outputs.is_empty()
    }
}
