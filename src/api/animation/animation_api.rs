use std::error::Error;

use crate::api::scenes::SceneDescriptor;
use crate::api::types::core::{Quality, RenderRequest};
use crate::core::animation::clip::AnimationClip;
use crate::core::animation::sequence::{FrameSequencer, SequenceResult};
use crate::core::animation::video::VideoExporter;
use crate::core::engine::rendering::renderer::types::RenderPreset;

use crate::api::engine::EngineApi;

impl EngineApi {
    /// Rend une animation complète à partir d'une scène de base et d'un clip.
    pub fn render_animation(
        &self,
        base: SceneDescriptor,
        clip: AnimationClip,
        request: &RenderRequest,
        frame_prefix: &str,
    ) -> Result<SequenceResult, Box<dyn Error>> {
        let preset = match request.quality {
            Quality::Preview => RenderPreset::AnimationFast,
            Quality::Hd => RenderPreset::UltraHdCpu,
            Quality::Production => RenderPreset::ProductionReference,
        };

        let sequencer = FrameSequencer::new(
            base,
            clip,
            request.output_dir.clone(),
            frame_prefix,
            preset,
            request.width,
            request.height,
        );

        sequencer.render_all()
    }

    /// Encode une séquence de frames en MP4/H.264.
    pub fn encode_animation_mp4<P: AsRef<std::path::Path>>(
        &self,
        sequence: &SequenceResult,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        VideoExporter::encode_from_result(sequence, output_path)
    }
}
