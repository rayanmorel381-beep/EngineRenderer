
use crate::core::coremanager::camera_manager::CameraManager;
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct CameraRig {
    manager: CameraManager,
}

impl CameraRig {
    pub fn cinematic(scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(Vec3::ZERO, scene_radius),
        }
    }

    pub fn focused_on(focus: Vec3, scene_radius: f64) -> Self {
        Self {
            manager: CameraManager::cinematic_for_scene(focus, scene_radius),
        }
    }

    pub fn reframe(&mut self, focus: Vec3, radius: f64) {
        self.manager.reframe(focus, radius);
    }

    pub fn build_camera(&self, aspect_ratio: f64, time: f64) -> Camera {
        self.manager.build_camera(aspect_ratio, time)
    }

    pub fn distance_to_focus(&self) -> f64 {
        self.manager.distance_to_focus()
    }

    pub fn inner(&self) -> &CameraManager {
        &self.manager
    }
}
