use std::{error::Error, path::PathBuf};

use crate::api::scene_descriptor::SceneDescriptor;
use crate::core::engine::rendering::renderer::{Renderer, types::RenderPreset};

use super::clip::AnimationClip;

#[derive(Debug, Clone)]
pub struct FrameResult {
    pub frame:       usize,
    pub time_secs:   f64,
    pub output_path: PathBuf,
    pub duration_ms: u128,
}

#[derive(Debug, Clone)]
pub struct SequenceResult {
    pub frames:      Vec<FrameResult>,
    pub total_ms:    u128,
    pub output_dir:  PathBuf,
    pub frame_count: usize,
    pub fps:         f64,
}

impl SequenceResult {
    pub fn average_frame_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.total_ms as f64 / self.frames.len() as f64
    }
}

pub struct FrameSequencer {
    pub base:         SceneDescriptor,
    pub clip:         AnimationClip,
    pub output_dir:   PathBuf,
    pub frame_prefix: String,
    pub preset:       RenderPreset,
    pub width:        usize,
    pub height:       usize,
}

impl FrameSequencer {
    pub fn new(
        base:         SceneDescriptor,
        clip:         AnimationClip,
        output_dir:   impl Into<PathBuf>,
        frame_prefix: impl Into<String>,
        preset:       RenderPreset,
        width:        usize,
        height:       usize,
    ) -> Self {
        Self {
            base,
            clip,
            output_dir: output_dir.into(),
            frame_prefix: frame_prefix.into(),
            preset,
            width,
            height,
        }
    }

    pub fn render_all(&self) -> Result<SequenceResult, Box<dyn Error>> {
        use crate::core::engine::acces_hardware::{precise_timestamp_ns, elapsed_ms as hw_elapsed};

        let frame_count = self.clip.frame_count();
        let renderer    = Renderer::with_resolution(self.width, self.height);
        let mut frames  = Vec::with_capacity(frame_count);
        let t_total     = precise_timestamp_ns();

        std::fs::create_dir_all(&self.output_dir)?;

        let ext = detect_ext(&self.frame_prefix);

        for idx in 0..frame_count {
            let time  = self.clip.time_for_frame(idx);
            let scene = self.clip.evaluate(&self.base, time);

            let file_name  = format!("{prefix}_{idx:05}.{ext}",
                prefix = stem(&self.frame_prefix),
                idx    = idx,
                ext    = ext,
            );
            let output_path = self.output_dir.join(&file_name);

            let (core_scene, camera) = scene.into_builder().build(self.width as f64 / self.height as f64);

            let t_frame = precise_timestamp_ns();
            let report  = renderer.render_scene_to_file(&core_scene, &camera, &output_path, self.preset)?;
            let frame_ms = hw_elapsed(t_frame, precise_timestamp_ns());

            eprintln!("animation: frame {}/{} t={:.3}s → {} ({:.1}ms)",
                idx + 1, frame_count, time, output_path.display(), frame_ms);

            frames.push(FrameResult {
                frame:       idx,
                time_secs:   time,
                output_path,
                duration_ms: report.duration_ms,
            });
        }

        let total_ms = hw_elapsed(t_total, precise_timestamp_ns()) as u128;

        Ok(SequenceResult {
            frames,
            total_ms,
            output_dir: self.output_dir.clone(),
            frame_count,
            fps: self.clip.fps,
        })
    }
}

fn detect_ext(prefix: &str) -> &str {
    if prefix.ends_with(".png") { "png" }
    else if prefix.ends_with(".exr") { "exr" }
    else { "png" }
}

fn stem(prefix: &str) -> &str {
    if let Some(dot) = prefix.rfind('.') { &prefix[..dot] } else { prefix }
}
