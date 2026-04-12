use crate::api::camera::controller::CameraController;

use super::engine_api::EngineApi;

// Re-export input facades
pub use crate::core::input::audio::mixer::Mixer;
pub use crate::core::input::camera::CameraRig;
pub use crate::core::input::events::EventLog;
pub use crate::core::input::manager::{InputDriver, InputMode};

impl EngineApi {
    // -- camera helpers -----------------------------------------------------

    pub fn camera(&self) -> CameraController {
        CameraController::new()
    }

    pub fn camera_cinematic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_cinematic(scene_radius)
    }

    pub fn camera_front(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_front(scene_radius)
    }

    pub fn camera_top_down(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_top_down(scene_radius)
    }

    pub fn camera_dramatic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_dramatic(scene_radius)
    }

    pub fn camera_closeup(&self, target: [f64; 3], distance: f64) -> CameraController {
        CameraController::preset_closeup(target, distance)
    }
}
