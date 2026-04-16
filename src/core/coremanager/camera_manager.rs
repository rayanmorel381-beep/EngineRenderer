use crate::core::engine::rendering::raytracing::{Camera, Vec3};

/// Gestionnaire caméra cinématique orienté scène.
#[derive(Debug, Clone, Copy)]
pub struct CameraManager {
    focus_point: Vec3,
    orbit_radius: f64,
    height: f64,
    vertical_fov: f64,
}

impl CameraManager {
    /// Crée un gestionnaire caméra initialisé pour cadrer une scène.
    pub fn cinematic_for_scene(focus_point: Vec3, scene_radius: f64) -> Self {
        let mut manager = Self {
            focus_point,
            orbit_radius: 10.0,
            height: 2.5,
            vertical_fov: 36.0,
        };
        manager.reframe(focus_point, scene_radius);
        manager
    }

    /// Recalcule l'orbite caméra en fonction du centre et du rayon de scène.
    pub fn reframe(&mut self, focus_point: Vec3, scene_radius: f64) {
        let safe_radius = scene_radius.max(1.0);
        self.focus_point = focus_point;
        self.orbit_radius = (safe_radius * 2.75).clamp(9.5, 48.0);
        self.height = (safe_radius * 0.74).clamp(2.2, 13.0);
        self.vertical_fov = (38.0 + safe_radius * 0.65).clamp(38.0, 54.0);
    }

    /// Construit une caméra physique animée pour l'instant `time`.
    pub fn build_camera(&self, aspect_ratio: f64, time: f64) -> Camera {
        let yaw = 0.25 + time * 0.45;
        let vertical_motion = (time * 0.8).sin() * 0.35;
        let origin = self.focus_point
            + Vec3::new(
                self.orbit_radius * yaw.cos(),
                self.height + vertical_motion,
                self.orbit_radius * yaw.sin(),
            );
        let motion_vector = Vec3::ZERO;
        let aperture_radius = (self.orbit_radius / 520.0).clamp(0.002, 0.010);

        Camera::look_at(
            origin,
            self.focus_point,
            Vec3::new(0.0, 1.0, 0.0),
            self.vertical_fov,
            aspect_ratio,
        )
        .with_physical_lens(aperture_radius, 0.0, motion_vector)
    }

    /// Retourne la distance caméra-centre de focus.
    pub fn distance_to_focus(&self) -> f64 {
        (self.orbit_radius * self.orbit_radius + self.height * self.height).sqrt()
    }
}
