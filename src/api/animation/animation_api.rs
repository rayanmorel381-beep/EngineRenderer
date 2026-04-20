use std::error::Error;

use crate::api::scenes::SceneDescriptor;
use crate::api::types::core::{Quality, RenderRequest};
use crate::core::animation::clip::AnimationClip;
use crate::core::animation::sequence::{FrameSequencer, SequenceResult};
use crate::core::animation::video::VideoExporter;
use crate::core::engine::rendering::renderer::types::RenderPreset;

use crate::api::engine::EngineApi;

impl EngineApi {
    /// Renders a sequence of animation frames from a base descriptor and clip.
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

    /// Encodes an existing rendered frame sequence into MP4.
    pub fn encode_animation_mp4<P: AsRef<std::path::Path>>(
        &self,
        sequence: &SequenceResult,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        VideoExporter::encode_from_result(sequence, output_path)
    }
}
