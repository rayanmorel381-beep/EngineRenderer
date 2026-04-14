use std::error::Error;

use crate::api::scenes::builder::SceneBuilder;
use crate::api::types::core::{Quality, RenderRequest, RenderResult};
use crate::core::engine::rendering::renderer::types::RenderPreset;
use crate::core::engine::rendering::renderer::Renderer;

use super::engine_api::EngineApi;

// Re-export rendering/framebuffer/postprocessing internals
pub use crate::core::engine::rendering::framebuffer::FrameBuffer;
pub use crate::core::engine::rendering::lod::manager::LodManager as RenderingLodManager;
pub use crate::core::engine::rendering::lod::tier::{LodThresholds, LodTier};
pub use crate::core::engine::rendering::postprocessing::blur::gaussian_weights;
pub use crate::core::engine::rendering::preprocessing::tone_mapping::ToneMappingOperator;
pub use crate::core::engine::rendering::utils::{
    aces_tonemap, barycentric, bias, cartesian_to_spherical, color_temperature,
    fresnel_dielectric, fresnel_schlick, fresnel_schlick_vec, gain, hsv_to_rgb,
    inverse_lerp, linear_to_srgb, luminance, quintic_smooth, reflect, reinhard_extended, remap,
    rgb_to_hsv, smoothstep, spherical_to_cartesian, srgb_to_linear, triangle_area, uncharted2_tonemap,
};

impl EngineApi {
    // -- render requests ----------------------------------------------------

    pub fn request_hd(&self) -> RenderRequest {
        RenderRequest::hd()
    }

    pub fn request_production(&self) -> RenderRequest {
        RenderRequest::production()
    }

    pub fn request_preview(&self) -> RenderRequest {
        RenderRequest::preview()
    }

    pub fn request_custom(
        &self,
        width: usize,
        height: usize,
        quality: Quality,
    ) -> RenderRequest {
        RenderRequest {
            width: width.clamp(64, 3840),
            height: height.clamp(64, 2160),
            quality,
            output_dir: std::path::PathBuf::from("output"),
            file_name: "api_render.ppm".to_string(),
        }
    }

    // -- rendering ----------------------------------------------------------

    pub fn render(
        &self,
        builder: SceneBuilder,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let (scene, camera) = builder.build(request.aspect_ratio());
        let renderer = Renderer::with_resolution(request.width, request.height);
        let preset = preset_for(request);
        let output_path = request.output_path();
        let report = renderer.render_scene_to_file(&scene, &camera, &output_path, preset)?;

        Ok(RenderResult {
            output_path,
            width: report.width,
            height: report.height,
            rendered_pixels: report.rendered_pixels,
            duration_ms: report.duration_ms,
            object_count: report.object_count,
            triangle_count: report.triangle_count,
            average_luminance: report.average_luminance,
            min_luminance: report.min_luminance,
            max_luminance: report.max_luminance,
            estimated_samples_per_pixel: report.estimated_samples_per_pixel,
        })
    }

    pub fn render_showcase(
        &self,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let renderer = Renderer::with_resolution(request.width, request.height);
        let preset = preset_for(request);
        let output_path = request.output_path();
        let report = renderer.render_to_file(&output_path, preset)?;

        Ok(RenderResult {
            output_path,
            width: report.width,
            height: report.height,
            rendered_pixels: report.rendered_pixels,
            duration_ms: report.duration_ms,
            object_count: report.object_count,
            triangle_count: report.triangle_count,
            average_luminance: report.average_luminance,
            min_luminance: report.min_luminance,
            max_luminance: report.max_luminance,
            estimated_samples_per_pixel: report.estimated_samples_per_pixel,
        })
    }
}

fn preset_for(request: &RenderRequest) -> RenderPreset {
    match request.quality {
        Quality::Preview => RenderPreset::PreviewCpu,
        Quality::Hd => RenderPreset::UltraHdCpu,
        Quality::Production => RenderPreset::ProductionReference,
    }
}
