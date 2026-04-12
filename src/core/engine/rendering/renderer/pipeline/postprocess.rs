//! Framebuffer operations, god rays, tone mapping & color grading.

use crate::core::engine::rendering::{
    effects::volumetric_effects::god_rays::GodRays,
    framebuffer::FrameBuffer,
    preprocessing::tone_mapping::ColorGrading,
    raytracing::shading::tone_map,
};

use super::super::Renderer;

impl Renderer {
    /// Applies a subtle depth-based fog tint to the framebuffer.
    /// Computes fog inline from depth values — no intermediate Vec allocation.
    pub(in crate::core::engine::rendering::renderer) fn apply_depth_fog(
        &self,
        framebuffer: &mut FrameBuffer,
    ) {
        let (depth_min, depth_max) = framebuffer.depth_range();
        let depth_span = (depth_max - depth_min).max(f64::EPSILON);
        let fog_color = crate::core::engine::rendering::raytracing::Vec3::new(0.6, 0.7, 0.85);
        let w = framebuffer.width;
        let h = framebuffer.height;
        for i in 0..(w * h) {
            let d = framebuffer.depth[i];
            let norm_depth = ((d - depth_min) / depth_span).clamp(0.0, 1.0);
            let fog_factor = (norm_depth * 0.03).min(0.03);
            framebuffer.color[i] = framebuffer.color[i] * (1.0 - fog_factor) + fog_color * fog_factor;
        }
    }

    /// Single god-ray pass with intensity driven by sun visibility.
    pub(in crate::core::engine::rendering::renderer) fn apply_god_rays(
        &self,
        framebuffer: &mut FrameBuffer,
        sun_screen_x: f64,
        intensity: f64,
    ) {
        // Single adaptive pass: sample count and exposure scale with intensity
        let god_rays = GodRays {
            num_samples: (40.0 + intensity * 60.0) as u32,  // 40..100 based on visibility
            density: 0.97,
            weight: 0.05,
            decay: 0.97,
            exposure: 0.08 * intensity,
        };
        god_rays.apply_to_buffer(
            &mut framebuffer.color,
            framebuffer.width,
            framebuffer.height,
            sun_screen_x.clamp(0.0, 1.0),
            0.35,
        );
    }

    pub(in crate::core::engine::rendering::renderer) fn apply_tone_mapping_and_grading(
        &self,
        framebuffer: &mut FrameBuffer,
    ) {
        let grading = ColorGrading::cinematic();
        let auto_exp = framebuffer.auto_exposure(0.18, -2.0, 4.0);
        framebuffer.apply_exposure(auto_exp);
        for pixel in &mut framebuffer.color {
            *pixel = tone_map(*pixel, 1.0);
            *pixel = grading.apply(*pixel);
        }
    }
}
