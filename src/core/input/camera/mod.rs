//! Public camera API for crate consumers.
//!
//! Provides [`CameraRig`] — a high-level orbital camera facade
//! with cinematic presets.  Delegates to the engine's `CameraManager`.

use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

/// High-level camera rig exposed to crate consumers.
#[derive(Debug, Clone, Copy)]
pub struct CameraRig {
    manager: CameraManager,
}

impl CameraRig {
    /// Default cinematic rig centred at the origin with the given scene radius.
    pub fn cinematic(scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(Vec3::ZERO, scene_radius),
        }
    }

    /// Create a rig focused on a specific point.
    pub fn focused_on(focus: Vec3, scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(focus, scene_radius),
        }
    }

    /// Reframe the camera to a new point / radius.
    pub fn reframe(&mut self, focus: Vec3, radius: f64) {
        self.manager.reframe(focus, radius);
    }

    /// Build a ray-tracing [`Camera`] for the given aspect ratio and time.
    pub fn build_camera(&self, aspect_ratio: f64, time: f64) -> Camera {
        self.manager.build_camera(aspect_ratio, time)
    }

    /// Distance from the camera orbit to the focus point.
    pub fn distance_to_focus(&self) -> f64 {
        self.manager.distance_to_focus()
    }

    /// Access the inner [`CameraManager`] for advanced use.
    pub fn inner(&self) -> &CameraManager {
        &self.manager
    }
}
