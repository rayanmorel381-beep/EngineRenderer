
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ResourceTracker {
    frame_outputs: Vec<PathBuf>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self { frame_outputs: Vec::new() }
    }

    pub fn record_output(&mut self, path: PathBuf) {
        self.frame_outputs.push(path);
    }

    pub fn output_count(&self) -> usize {
        self.frame_outputs.len()
    }

    pub fn outputs(&self) -> &[PathBuf] {
        &self.frame_outputs
    }

    pub fn has_outputs(&self) -> bool {
        !self.frame_outputs.is_empty()
    }
}
