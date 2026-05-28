//! Preset → `RenderConfig` mapping and worker thread accounting.

use crate::core::engine::rendering::raytracing::RenderConfig;

use super::state::Renderer;
use super::types::RenderPreset;

impl Renderer {
    pub fn worker_threads(&self) -> usize {
        self.hw_caps
            .optimal_render_threads_for_input(self.width.saturating_mul(self.height))
    }

    pub fn config_for(&self, preset: RenderPreset) -> RenderConfig {
        let (cfg_w, cfg_h) = match preset {
            RenderPreset::AnimationFast => (self.width, self.height),
            RenderPreset::PreviewCpu => (self.width.min(1920), self.height.min(1080)),
            RenderPreset::UltraHdCpu => (self.width, self.height),
            RenderPreset::ProductionReference => (self.width, self.height),
        };
        let max_threads = self
            .hw_caps
            .optimal_render_threads_for_input(cfg_w.saturating_mul(cfg_h));
        let threads = self.worker_threads().min(max_threads).max(1);
        match preset {
            RenderPreset::AnimationFast => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 1,
                max_bounces: 1,
                max_distance: 160.0,
                thread_count: threads,
                denoise_strength: 0.0,
                adaptive_sampling: false,
                firefly_threshold: 3.0,
                denoise_radius: 2,
            },
            RenderPreset::PreviewCpu => RenderConfig {
                width: cfg_w,
                height: cfg_h,
                base_samples_per_pixel: 4,
                max_bounces: 1,
                max_distance: 480.0,
                thread_count: threads,
                denoise_strength: 0.10,
                adaptive_sampling: true,
                firefly_threshold: 2.20,
                denoise_radius: 1,
            },
            RenderPreset::UltraHdCpu => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 16,
                max_bounces: 3,
                max_distance: 1200.0,
                thread_count: threads,
                denoise_strength: 0.12,
                adaptive_sampling: true,
                firefly_threshold: 1.90,
                denoise_radius: 1,
            },
            RenderPreset::ProductionReference => RenderConfig {
                width: self.width,
                height: self.height,
                base_samples_per_pixel: 32,
                max_bounces: 4,
                max_distance: 1500.0,
                thread_count: threads,
                denoise_strength: 0.14,
                adaptive_sampling: true,
                firefly_threshold: 1.60,
                denoise_radius: 1,
            },
        }
    }
}
