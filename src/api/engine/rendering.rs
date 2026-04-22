use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

use crate::api::animation::AnimationClip;
use crate::api::scenes::builder::SceneBuilder;
use crate::api::scenes::SceneDescriptor;
use crate::api::types::core::{Quality, RenderRequest, RenderResult};
use crate::core::animation::sequence::{FrameSequencer, SequenceResult};
use crate::core::coremanager::engine_manager::Engine;
use crate::core::engine::rendering::renderer::types::RenderPreset;
use crate::core::engine::rendering::renderer::Renderer;

use super::engine_api::EngineApi;

pub use crate::core::engine::rendering::framebuffer::FrameBuffer;
pub use crate::core::engine::rendering::lod::manager::LodManager as RenderingLodManager;
pub use crate::core::engine::rendering::lod::tier::{LodThresholds, LodTier};
pub use crate::core::engine::rendering::postprocessing::blur::gaussian_weights;
pub use crate::core::engine::rendering::preprocessing::tone_mapping::ToneMappingOperator;
pub use crate::core::engine::rendering::raytracing::Vec3;
pub use crate::core::engine::rendering::utils::{
    aces_tonemap, barycentric, bias, cartesian_to_spherical, color_temperature,
    fresnel_dielectric, fresnel_schlick, fresnel_schlick_vec, gain, hsv_to_rgb,
    inverse_lerp, linear_to_srgb, luminance, quintic_smooth, reflect, reinhard_extended, remap,
    rgb_to_hsv, smoothstep, spherical_to_cartesian, srgb_to_linear, triangle_area, uncharted2_tonemap,
};

/// Realtime run request.
#[derive(Debug, Clone)]
pub struct RealtimeRequest {
    /// Target output width.
    pub width: u32,
    /// Target output height.
    pub height: u32,
    /// Requested frames per second.
    pub target_fps: u32,
    /// Requested run duration in seconds.
    pub duration_seconds: u32,
}

impl Default for RealtimeRequest {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            target_fps: 120,
            duration_seconds: 10,
        }
    }
}

/// Realtime run result summary.
#[derive(Debug, Clone)]
pub struct RealtimeResult {
    /// Effective output width.
    pub width: u32,
    /// Effective output height.
    pub height: u32,
    /// Effective target FPS used by the run.
    pub target_fps: u32,
    /// Effective run duration in seconds.
    pub duration_seconds: u32,
    /// Wall-clock elapsed time in milliseconds.
    pub elapsed_ms: f64,
}

impl EngineApi {
    // -- render requests ----------------------------------------------------

    /// Returns a standard HD render request.
    pub fn request_hd(&self) -> RenderRequest {
        RenderRequest::hd()
    }

    /// Returns a production render request.
    pub fn request_production(&self) -> RenderRequest {
        RenderRequest::production()
    }

    /// Returns a preview render request.
    pub fn request_preview(&self) -> RenderRequest {
        RenderRequest::preview()
    }

    /// Builds a custom render request with clamped resolution.
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

    /// Renders a scene built through [`SceneBuilder`].
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

    /// Renders the engine showcase scene.
    pub fn render_showcase(
        &self,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let preset = preset_for(request);
        let renderer = match (preset, request.width, request.height) {
            (RenderPreset::PreviewCpu, 1920, 1080) => Renderer::from_preset(preset),
            (RenderPreset::UltraHdCpu, 2560, 1440) => Renderer::from_preset(preset),
            (RenderPreset::ProductionReference, 3840, 2160) => Renderer::from_preset(preset),
            _ => Renderer::with_resolution(request.width, request.height),
        };
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

    /// Runs realtime rendering with explicit runtime settings.
    pub fn render_realtime(
        &self,
        request: &RealtimeRequest,
    ) -> Result<RealtimeResult, Box<dyn Error>> {
        let width = request.width.max(1) as usize;
        let height = request.height.max(1) as usize;
        let fps = request.target_fps.max(1);
        let seconds = request.duration_seconds.max(1);

        let start = Instant::now();
        Engine::realtime_with_resolution(width, height).run_realtime(seconds, fps)?;
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(RealtimeResult {
            width: width as u32,
            height: height as u32,
            target_fps: fps,
            duration_seconds: seconds,
            elapsed_ms,
        })
    }

    /// Returns the default realtime HD profile.
    pub fn realtime_hd(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 1920,
            height: 1080,
            target_fps: 120,
            duration_seconds: 10,
        }
    }

    /// Returns a lightweight mobile realtime profile.
    pub fn realtime_mobile(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 640,
            height: 360,
            target_fps: 120,
            duration_seconds: 8,
        }
    }

    /// Returns an ultra-resolution realtime profile.
    pub fn realtime_ultra(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 3840,
            height: 2160,
            target_fps: 120,
            duration_seconds: 5,
        }
    }
}

fn preset_for(request: &RenderRequest) -> RenderPreset {
    match request.quality {
        Quality::Preview => RenderPreset::PreviewCpu,
        Quality::Hd => RenderPreset::UltraHdCpu,
        Quality::Production => RenderPreset::ProductionReference,
    }
}

/// Video generation request.
#[derive(Debug, Clone)]
pub struct GeneratorRequest {
    /// Output frame width.
    pub width: usize,
    /// Output frame height.
    pub height: usize,
    /// Quality profile used during rendering.
    pub quality: Quality,
    /// Directory where frame files are generated.
    pub output_dir: PathBuf,
    /// Final MP4 output path.
    pub output_mp4: PathBuf,
    /// Prefix used for generated frame queue names.
    pub frame_prefix: String,
}

impl GeneratorRequest {
    /// Returns the preview generation profile.
    pub fn preview() -> Self {
        Self {
            width: 1280,
            height: 720,
            quality: Quality::Preview,
            output_dir: PathBuf::from("output/video"),
            output_mp4: PathBuf::from("output/video/animation.mp4"),
            frame_prefix: String::from("frame"),
        }
    }

    /// Converts this generator request to a render request.
    pub fn to_render_request(&self) -> RenderRequest {
        RenderRequest::preview()
            .with_quality(self.quality)
            .with_resolution(self.width, self.height)
            .with_output(self.output_dir.clone(), String::from("frame.ppm"))
    }

    /// Converts this generator request to a renderer preset.
    pub fn to_preset(&self) -> RenderPreset {
        match self.quality {
            Quality::Preview => RenderPreset::AnimationFast,
            Quality::Hd => RenderPreset::UltraHdCpu,
            Quality::Production => RenderPreset::ProductionReference,
        }
    }
}

/// Renders and encodes a full animation sequence.
pub fn generate_video(
    base: SceneDescriptor,
    clip: AnimationClip,
    request: &GeneratorRequest,
) -> Result<SequenceResult, Box<dyn Error>> {
    let api = EngineApi::new();
    let sequence = api.render_animation(
        base,
        clip,
        &request.to_render_request(),
        &request.frame_prefix,
    )?;
    api.encode_animation_mp4(&sequence, &request.output_mp4)?;
    Ok(sequence)
}

/// Renders an animation sequence for live preview.
pub fn preview_window(
    base: SceneDescriptor,
    clip: AnimationClip,
    request: &GeneratorRequest,
) -> Result<SequenceResult, Box<dyn Error>> {
    let sequencer = FrameSequencer::new(
        base,
        clip,
        request.output_dir.clone(),
        &request.frame_prefix,
        request.to_preset(),
        request.width,
        request.height,
    );
    sequencer.render_all_to_window()
}
