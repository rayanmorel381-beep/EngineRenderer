use std::error::Error;

use crate::api::scenes::builder::SceneBuilder;
use crate::api::types::{Quality, RenderRequest, RenderResult};
use crate::core::engine::rendering::renderer::types::RenderPreset;
use crate::core::engine::rendering::renderer::Renderer;

use super::capabilities;
use super::prompt;

/// AI-facing facade for the rendering engine.
///
/// This is the main entry point for any AI agent, LLM tool, or automated
/// pipeline that wants to produce images. Every method accepts and returns
/// simple, serialisable types so they can travel over JSON, MCP, or any
/// other wire format.
///
/// # Quick start
///
/// ```ignore
/// use enginerenderer::api::ai::AiRenderer;
/// use enginerenderer::api::types::RenderRequest;
///
/// let ai = AiRenderer::new();
/// let request = RenderRequest::hd();
/// let result = ai.render_scene_builder(
///     ai.scene_from_prompt("a blue ocean planet orbiting a yellow star"),
///     &request,
/// )?;
/// ```
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
    pub fn new() -> Self {
        Self {
            preset_hd: RenderPreset::UltraHdCpu,
            preset_production: RenderPreset::ProductionReference,
        }
    }

    /// Render a scene described by a [`SceneBuilder`] with the given request
    /// parameters. Returns a [`RenderResult`] with paths and quality metrics.
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

    /// Render the built-in realistic showcase scene.
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

    /// Build a [`SceneBuilder`] from a natural-language prompt.
    pub fn scene_from_prompt(&self, prompt: &str) -> SceneBuilder {
        prompt::scene_from_prompt(prompt)
    }

    /// List every capability the engine exposes.
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
