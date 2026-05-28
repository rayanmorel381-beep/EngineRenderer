use crate::api::camera::controller::CameraController;

use super::engine_api::EngineApi;

/// Audio mixing helper re-export.
pub use crate::core::input::audio::mixer::Mixer;
/// Runtime camera rig re-export.
pub use crate::core::input::camera::CameraRig;
/// Event log re-export.
pub use crate::core::input::events::EventLog;
/// Input mode and driver re-exports.
pub use crate::core::input::manager::{InputDriver, InputMode};

impl EngineApi {
    // -- camera helpers -----------------------------------------------------

    /// Returns a new camera controller.
    pub fn camera(&self) -> CameraController {
        CameraController::new()
    }

    /// Returns the cinematic camera preset for a scene radius.
    pub fn camera_cinematic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_cinematic(scene_radius)
    }

    /// Returns the front camera preset for a scene radius.
    pub fn camera_front(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_front(scene_radius)
    }

    /// Returns the top-down camera preset for a scene radius.
    pub fn camera_top_down(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_top_down(scene_radius)
    }

    /// Returns the dramatic camera preset for a scene radius.
    pub fn camera_dramatic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_dramatic(scene_radius)
    }

    /// Returns the closeup preset focused on a target.
    pub fn camera_closeup(&self, target: [f64; 3], distance: f64) -> CameraController {
        CameraController::preset_closeup(target, distance)
    }
}
