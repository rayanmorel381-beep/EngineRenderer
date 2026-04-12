//! Engine configuration module.

use std::path::PathBuf;

use crate::core::engine::rendering::renderer::types::RenderPreset;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub width: usize,
    pub height: usize,
    pub output_path: PathBuf,
    pub render_preset: RenderPreset,
}

impl EngineConfig {
    pub fn ultra_hd_cpu() -> Self {
        Self {
            width: 2560,
            height: 1440,
            output_path: PathBuf::from("output/render_output.ppm"),
            render_preset: RenderPreset::UltraHdCpu,
        }
    }

    pub fn production_reference() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_path: PathBuf::from("output/render_output.ppm"),
            render_preset: RenderPreset::ProductionReference,
        }
    }

    /// Minimal config for integration tests — tiny resolution to avoid
    /// blowing up CPU/RAM.
    pub fn test_minimal() -> Self {
        Self {
            width: 80,
            height: 45,
            output_path: PathBuf::from("output/render_test.ppm"),
            render_preset: RenderPreset::PreviewCpu,
        }
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::ultra_hd_cpu()
    }
}
