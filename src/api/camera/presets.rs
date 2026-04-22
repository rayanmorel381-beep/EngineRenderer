use crate::api::camera::CameraController;

impl CameraController {
    /// Returns a front-facing framing preset.
    pub fn preset_front(scene_radius: f64) -> Self {
        let dist = scene_radius.max(1.0) * 2.8;
        Self::new().look_at([0.0, dist * 0.3, dist], [0.0, 0.0, 0.0]).fov(38.0)
    }

    /// Returns a top-down framing preset.
    pub fn preset_top_down(scene_radius: f64) -> Self {
        let dist = scene_radius.max(1.0) * 3.0;
        Self::new().look_at([0.0, dist, 0.01], [0.0, 0.0, 0.0]).fov(42.0)
    }

    /// Returns a dramatic angled framing preset.
    pub fn preset_dramatic(scene_radius: f64) -> Self {
        let dist = scene_radius.max(1.0) * 2.5;
        Self::new().look_at([dist * 0.8, dist * 0.15, dist * 0.6], [0.0, 0.0, 0.0]).fov(32.0)
    }

    /// Returns a cinematic framing preset with aperture.
    pub fn preset_cinematic(scene_radius: f64) -> Self {
        let dist = scene_radius.max(1.0) * 2.5;
        Self::new()
            .look_at(
                [dist * 0.7, dist * 0.45, dist * 0.7],
                [0.0, 0.0, 0.0],
            )
            .fov(42.0)
            .aperture(0.035)
    }

    /// Returns a close-up preset focused on a target point.
    pub fn preset_closeup(target: [f64; 3], approach_distance: f64) -> Self {
        let dist = approach_distance.max(0.5);
        Self::new()
            .look_at(
                [target[0] + dist * 0.6, target[1] + dist * 0.3, target[2] + dist * 0.8],
                target,
            )
            .fov(28.0)
            .aperture(0.05)
    }
}
