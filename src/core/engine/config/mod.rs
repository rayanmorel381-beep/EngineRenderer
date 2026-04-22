use std::path::PathBuf;

use crate::core::engine::rendering::renderer::types::RenderPreset;

/// Global engine configuration for offline and realtime rendering.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Output image width in pixels.
    pub width: usize,
    /// Output image height in pixels.
    pub height: usize,
    /// Destination path for the rendered image.
    pub output_path: PathBuf,
    /// Rendering quality and feature preset.
    pub render_preset: RenderPreset,
}

impl EngineConfig {
    fn default_output_path(file_name: &str) -> PathBuf {
        #[cfg(target_os = "android")]
        {
            let mut path = std::env::temp_dir();
            path.push("enginerenderer-output");
            path.push(file_name);
            return path;
        }
        #[allow(unreachable_code)]
        PathBuf::from("output").join(file_name)
    }

    /// Returns a high-quality CPU preset for ultra-HD rendering.
    pub fn ultra_hd_cpu() -> Self {
        Self {
            width: 2560,
            height: 1440,
            output_path: Self::default_output_path("render_output.ppm"),
            render_preset: RenderPreset::UltraHdCpu,
        }
    }

    /// Returns a production-grade reference preset.
    pub fn production_reference() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_path: Self::default_output_path("render_output.ppm"),
            render_preset: RenderPreset::ProductionReference,
        }
    }

    /// Returns a low-latency preset intended for realtime preview.
    pub fn realtime_preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            output_path: Self::default_output_path("realtime_preview.ppm"),
            render_preset: RenderPreset::AnimationFast,
        }
    }

    /// Returns a tiny preset suitable for quick tests.
    pub fn test_minimal() -> Self {
        Self {
            width: 80,
            height: 45,
            output_path: Self::default_output_path("render_test.ppm"),
            render_preset: RenderPreset::PreviewCpu,
        }
    }

    /// Returns the image aspect ratio as width divided by height.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::ultra_hd_cpu()
    }
}
