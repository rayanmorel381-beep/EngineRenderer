//! Frame-rate loop controller — adaptive quality budgeting.

#[derive(Debug, Clone, Copy)]
pub struct FrameTarget {
    pub target_frame_ms: f64,
    pub quality_bias: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopController {
    target_fps: f64,
    max_substeps: u32,
}

impl LoopController {
    pub fn new(target_fps: f64, max_substeps: u32) -> Self {
        Self {
            target_fps: target_fps.max(1.0),
            max_substeps: max_substeps.max(1),
        }
    }

    pub fn frame_target(&self, width: usize, height: usize, detail_scale: f64) -> FrameTarget {
        let load_factor = (width * height) as f64 / (1920 * 1080) as f64;
        let quality_bias = (1.15 / (load_factor * detail_scale.max(1.0))).clamp(0.65, 1.15);

        FrameTarget {
            target_frame_ms: 1000.0 / self.target_fps,
            quality_bias,
        }
    }

    pub fn recommended_substeps(&self, quality_bias: f64, requested: u32) -> u32 {
        ((requested as f64 * quality_bias).round() as u32).clamp(1, self.max_substeps)
    }
}
