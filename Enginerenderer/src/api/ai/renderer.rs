use std::error::Error;

use crate::api::scenes::builder::SceneBuilder;
use crate::api::types::{Quality, RenderRequest, RenderResult};
use crate::core::engine::rendering::renderer::types::RenderPreset;
use crate::core::engine::rendering::renderer::Renderer;

use super::capabilities;
use super::prompt;

/// High-level AI rendering facade.
#[derive(Debug)]
pub struct AiRenderer {
    preset_hd: RenderPreset,
    preset_production: RenderPreset,
}

impl Default for AiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl AiRenderer {
    /// Creates a renderer with default HD and production presets.
    pub fn new() -> Self {
        Self {
            preset_hd: RenderPreset::UltraHdCpu,
            preset_production: RenderPreset::ProductionReference,
        }
    }

    /// Renders a scene built from a [`SceneBuilder`] and request parameters.
    pub fn render_scene_builder(
        &self,
        builder: SceneBuilder,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let (scene, camera) = builder.build(request.aspect_ratio());
        let renderer = Renderer::with_resolution(request.width, request.height);
        let preset = self.preset_for(request);
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

    /// Renders the built-in showcase scene.
    pub fn render_showcase(&self, request: &RenderRequest) -> Result<RenderResult, Box<dyn Error>> {
        let renderer = Renderer::with_resolution(request.width, request.height);
        let preset = self.preset_for(request);
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

    /// Builds a scene from a text prompt and renders it.
    pub fn render_prompt(
        &self,
        prompt: &str,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let builder = self.scene_from_prompt(prompt).auto_frame();
        self.render_scene_builder(builder, request)
    }

    /// Converts a text prompt into a scene builder.
    pub fn scene_from_prompt(&self, prompt: &str) -> SceneBuilder {
        prompt::scene_from_prompt(prompt)
    }

    /// Returns runtime capabilities available to the AI renderer.
    pub fn capabilities(&self) -> capabilities::Capabilities {
        capabilities::discover()
    }

    fn preset_for(&self, request: &RenderRequest) -> RenderPreset {
        match request.quality {
            Quality::Preview => RenderPreset::PreviewCpu,
            Quality::Hd => self.preset_hd,
            Quality::Production => self.preset_production,
        }
    }
}
