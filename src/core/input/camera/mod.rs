
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

#[derive(Debug, Clone, Copy)]
/// Runtime camera rig facade around `CameraManager`.
pub struct CameraRig {
    manager: CameraManager,
}

impl CameraRig {
    /// Creates a cinematic rig centered at origin for a scene radius.
    pub fn cinematic(scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(Vec3::ZERO, scene_radius),
        }
    }

    /// Creates a cinematic rig focused on a specific point.
    pub fn focused_on(focus: Vec3, scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(focus, scene_radius),
        }
    }

    /// Reframes the rig using a focus point and radius.
    pub fn reframe(&mut self, focus: Vec3, radius: f64) {
        self.manager.reframe(focus, radius);
    }

    /// Builds a camera for the given aspect ratio and timeline time.
    pub fn build_camera(&self, aspect_ratio: f64, time: f64) -> Camera {
        self.manager.build_camera(aspect_ratio, time)
    }

    /// Returns distance from camera to focus point.
    pub fn distance_to_focus(&self) -> f64 {
        self.manager.distance_to_focus()
    }

    /// Returns the inner camera manager reference.
    pub fn inner(&self) -> &CameraManager {
        &self.manager
    }
}
