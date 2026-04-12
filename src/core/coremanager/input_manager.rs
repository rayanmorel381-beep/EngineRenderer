#[derive(Debug, Clone, Copy)]
pub struct FrameInput {
    pub orbit_bias: f64,
    pub exposure_nudge: f64,
    pub time_scale: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct InputManager {
    cinematic_mode: bool,
}

impl InputManager {
    pub fn new(cinematic_mode: bool) -> Self {
        Self { cinematic_mode }
    }

    pub fn sample_cinematic_input(&self, time: f64) -> FrameInput {
        if !self.cinematic_mode {
            return FrameInput {
                orbit_bias: 0.0,
                exposure_nudge: 0.0,
                time_scale: 1.0,
            };
        }

        FrameInput {
            orbit_bias: (time * 0.7).sin() * 0.35,
            exposure_nudge: (time * 0.45).cos() * 0.04,
            time_scale: (1.0 + (time * 0.2).sin() * 0.08).clamp(0.92, 1.08),
        }
    }
}
