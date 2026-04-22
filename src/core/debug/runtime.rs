#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeAdaptationState {
    pub target_frame_ms: f64,
    pub frame_p50_ms: f64,
    pub frame_p95_ms: f64,
    pub frame_p99_ms: f64,
    pub jitter_ms: f64,
    pub quality_bias: f64,
    pub sample_pressure_scale: f64,
    pub scheduler_granularity: f64,
    pub substeps: u32,
    pub internal_width: usize,
    pub internal_height: usize,
    pub output_width: usize,
    pub output_height: usize,
    pub resize_cooldown_frames: usize,
    pub over_budget_streak: usize,
    pub under_budget_streak: usize,
}

impl Default for RuntimeAdaptationState {
    fn default() -> Self {
        Self {
            target_frame_ms: 0.0,
            frame_p50_ms: 0.0,
            frame_p95_ms: 0.0,
            frame_p99_ms: 0.0,
            jitter_ms: 0.0,
            quality_bias: 1.0,
            sample_pressure_scale: 1.0,
            scheduler_granularity: 1.0,
            substeps: 1,
            internal_width: 0,
            internal_height: 0,
            output_width: 0,
            output_height: 0,
            resize_cooldown_frames: 0,
            over_budget_streak: 0,
            under_budget_streak: 0,
        }
    }
}

impl RuntimeAdaptationState {
    pub fn internal_scale(&self) -> f64 {
        if self.output_width == 0 || self.output_height == 0 {
            return 1.0;
        }

        let scale_x = self.internal_width as f64 / self.output_width as f64;
        let scale_y = self.internal_height as f64 / self.output_height as f64;
        scale_x.min(scale_y)
    }

    pub fn budget_pressure(&self) -> f64 {
        if self.target_frame_ms <= 0.0 {
            return 0.0;
        }

        ((self.frame_p95_ms / self.target_frame_ms) - 1.0).max(0.0)
    }

    pub fn tail_pressure(&self) -> f64 {
        if self.target_frame_ms <= 0.0 {
            return 0.0;
        }

        ((self.frame_p99_ms / self.target_frame_ms) - 1.0).max(0.0)
    }
}