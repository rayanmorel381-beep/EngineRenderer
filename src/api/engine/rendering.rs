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
pub use crate::core::engine::rendering::utils::{
    aces_tonemap, barycentric, bias, cartesian_to_spherical, color_temperature,
    fresnel_dielectric, fresnel_schlick, fresnel_schlick_vec, gain, hsv_to_rgb,
    inverse_lerp, linear_to_srgb, luminance, quintic_smooth, reflect, reinhard_extended, remap,
    rgb_to_hsv, smoothstep, spherical_to_cartesian, srgb_to_linear, triangle_area, uncharted2_tonemap,
};

/// Paramètres d'une session de rendu temps réel.
#[derive(Debug, Clone)]
pub struct RealtimeRequest {
    /// Largeur de la fenêtre en pixels.
    pub width: u32,
    /// Hauteur de la fenêtre en pixels.
    pub height: u32,
    /// Fréquence d'images cible en Hz.
    pub target_fps: u32,
    /// Durée de la session en secondes.
    pub duration_seconds: u32,
}

impl Default for RealtimeRequest {
    fn default() -> Self {
        Self {
            width: 640,
            height: 360,
            target_fps: 60,
            duration_seconds: 8,
        }
    }
}

/// Résultat d'une session de rendu temps réel.
#[derive(Debug, Clone)]
pub struct RealtimeResult {
    /// Largeur effective en pixels.
    pub width: u32,
    /// Hauteur effective en pixels.
    pub height: u32,
    /// Fréquence d'images cible en Hz.
    pub target_fps: u32,
    /// Durée de la session en secondes.
    pub duration_seconds: u32,
    /// Durée réelle écoulée en millisecondes.
    pub elapsed_ms: f64,
}

impl EngineApi {
    // -- render requests ----------------------------------------------------

    /// Retourne une `RenderRequest` en qualité HD (1920×1080).
    pub fn request_hd(&self) -> RenderRequest {
        RenderRequest::hd()
    }

    /// Retourne une `RenderRequest` en qualité production (3840×2160).
    pub fn request_production(&self) -> RenderRequest {
        RenderRequest::production()
    }

    /// Retourne une `RenderRequest` en qualité préview (1280×720).
    pub fn request_preview(&self) -> RenderRequest {
        RenderRequest::preview()
    }

    /// Retourne une `RenderRequest` avec les dimensions et la qualité spécifiées.
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

    /// Rend une scène construite par un `SceneBuilder`.
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

    /// Rend la scène de démonstration intégrée.
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

    /// Lance une session de rendu temps réel dans une fenêtre native.
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

    /// Retourne une `RealtimeRequest` HD (1920×1080 @ 60 fps, 10 s).
    pub fn realtime_hd(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 1920,
            height: 1080,
            target_fps: 60,
            duration_seconds: 10,
        }
    }

    /// Retourne une `RealtimeRequest` mobile (640×360 @ 120 fps, 8 s).
    pub fn realtime_mobile(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 640,
            height: 360,
            target_fps: 120,
            duration_seconds: 8,
        }
    }

    /// Retourne une `RealtimeRequest` ultra HD (3840×2160 @ 30 fps, 5 s).
    pub fn realtime_ultra(&self) -> RealtimeRequest {
        RealtimeRequest {
            width: 3840,
            height: 2160,
            target_fps: 30,
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

/// Paramètres de génération d'une animation (frames + encodage MP4).
#[derive(Debug, Clone)]
pub struct GeneratorRequest {
    /// Largeur des frames en pixels.
    pub width: usize,
    /// Hauteur des frames en pixels.
    pub height: usize,
    /// Qualité de rendu.
    pub quality: Quality,
    /// Répertoire de sortie pour les frames individuelles.
    pub output_dir: PathBuf,
    /// Chemin du fichier MP4 final.
    pub output_mp4: PathBuf,
    /// Préfixe des noms de fichier des frames (ex. `"frame"` → `frame_0001.ppm`).
    pub frame_prefix: String,
}

impl GeneratorRequest {
    /// Crée une requête de génération en qualité préview (1280×720).
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

    /// Convertit en [`RenderRequest`] compatible avec le pipeline de rendu.
    pub fn to_render_request(&self) -> RenderRequest {
        RenderRequest::preview()
            .with_quality(self.quality)
            .with_resolution(self.width, self.height)
            .with_output(self.output_dir.clone(), String::from("frame.ppm"))
    }

    /// Retourne le [`RenderPreset`] correspondant à la qualité demandée.
    pub fn to_preset(&self) -> RenderPreset {
        match self.quality {
            Quality::Preview => RenderPreset::AnimationFast,
            Quality::Hd => RenderPreset::UltraHdCpu,
            Quality::Production => RenderPreset::ProductionReference,
        }
    }
}

/// Rend toutes les frames de l'animation et les encode en MP4.
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

/// Rend l'animation frame par frame en l'affichant dans une fenêtre de prévisualisation.
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
