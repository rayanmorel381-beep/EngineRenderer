use crate::api::camera::controller::CameraController;

use super::engine_api::EngineApi;

pub use crate::core::input::audio::mixer::Mixer;
pub use crate::core::input::camera::CameraRig;
pub use crate::core::input::events::EventLog;
pub use crate::core::input::manager::{InputDriver, InputMode};

impl EngineApi {
    // -- camera helpers -----------------------------------------------------

    /// Retourne un nouveau contrôleur de caméra vide.
    pub fn camera(&self) -> CameraController {
        CameraController::new()
    }

    /// Caméra cinématique pré-positionnée autour d'une scène de rayon `scene_radius`.
    pub fn camera_cinematic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_cinematic(scene_radius)
    }

    /// Caméra frontale pré-positionnée autour d'une scène de rayon `scene_radius`.
    pub fn camera_front(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_front(scene_radius)
    }

    /// Caméra vue du dessus pré-positionnée autour d'une scène de rayon `scene_radius`.
    pub fn camera_top_down(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_top_down(scene_radius)
    }

    /// Caméra dramatique (angle bas) pré-positionnée autour d'une scène de rayon `scene_radius`.
    pub fn camera_dramatic(&self, scene_radius: f64) -> CameraController {
        CameraController::preset_dramatic(scene_radius)
    }

    /// Caméra rapprochée pointant vers `target` à `distance` unités.
    pub fn camera_closeup(&self, target: [f64; 3], distance: f64) -> CameraController {
        CameraController::preset_closeup(target, distance)
    }
}
