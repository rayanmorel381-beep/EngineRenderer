#[derive(Debug, Clone, Copy)]
pub struct FrameStep {
    pub frame_index: u64,
    pub delta_seconds: f64,
    pub integration_steps: u32,
    pub absolute_time: f64,
}

#[derive(Debug, Clone)]
pub struct TimeManager {
    frame_index: u64,
    accumulated_seconds: f64,
    minimum_delta: f64,
}

impl TimeManager {
    pub fn new(minimum_delta: f64) -> Self {
        Self {
            frame_index: 0,
            accumulated_seconds: 0.0,
            minimum_delta: minimum_delta.max(0.001),
        }
    }

    pub fn advance_frame(&mut self, simulation_seconds: f64, integration_steps: u32) -> FrameStep {
        self.frame_index += 1;
        let delta_seconds = simulation_seconds.max(self.minimum_delta);
        self.accumulated_seconds += delta_seconds;

        FrameStep {
            frame_index: self.frame_index,
            delta_seconds,
            integration_steps: integration_steps.max(1),
            absolute_time: self.accumulated_seconds,
        }
    }
}
