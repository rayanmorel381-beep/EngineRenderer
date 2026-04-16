//! Engine configuration module.

use std::path::PathBuf;

use crate::core::engine::rendering::renderer::types::RenderPreset;

/// Configuration runtime principale du moteur de rendu.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Largeur cible du rendu.
    pub width: usize,
    /// Hauteur cible du rendu.
    pub height: usize,
    /// Chemin de sortie image principal.
    pub output_path: PathBuf,
    /// Preset de rendu utilisé.
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

    /// Preset Ultra HD orienté qualité CPU.
    pub fn ultra_hd_cpu() -> Self {
        Self {
            width: 2560,
            height: 1440,
            output_path: Self::default_output_path("render_output.ppm"),
            render_preset: RenderPreset::UltraHdCpu,
        }
    }

    /// Preset référence production (résolution maximale prévue).
    pub fn production_reference() -> Self {
        Self {
            width: 3840,
            height: 2160,
            output_path: Self::default_output_path("render_output.ppm"),
            render_preset: RenderPreset::ProductionReference,
        }
    }

    /// Preset temps réel pour aperçu interactif.
    pub fn realtime_preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            output_path: Self::default_output_path("realtime_preview.ppm"),
            render_preset: RenderPreset::AnimationFast,
        }
    }

    /// Minimal config for integration tests — tiny resolution to avoid
    /// blowing up CPU/RAM.
    pub fn test_minimal() -> Self {
        Self {
            width: 80,
            height: 45,
            output_path: Self::default_output_path("render_test.ppm"),
            render_preset: RenderPreset::PreviewCpu,
        }
    }

    /// Retourne le ratio d'aspect `width / height`.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self::ultra_hd_cpu()
    }
}
