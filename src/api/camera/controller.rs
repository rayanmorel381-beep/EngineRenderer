use crate::api::types::CameraDesc;
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

/// AI-friendly camera controller.
///
/// Wraps the low-level [`Camera`] with simple high-level operations:
/// orbit, look-at, auto-frame. All inputs are plain arrays/scalars.
#[derive(Debug, Clone, Copy, Default)]
pub struct CameraController {
    desc: CameraDesc,
}

impl CameraController {
    /// Crée un `CameraController` avec les valeurs par défaut.
    pub fn new() -> Self {
        Self::default()
    }

    /// Place the camera at `eye` looking at `target`.
    pub fn look_at(mut self, eye: [f64; 3], target: [f64; 3]) -> Self {
        self.desc.eye = eye;
        self.desc.target = target;
        self
    }

    /// Set vertical field of view in degrees (clamped 10–120).
    pub fn fov(mut self, degrees: f64) -> Self {
        self.desc.fov_degrees = degrees.clamp(10.0, 120.0);
        self
    }

    /// Set lens aperture radius for depth-of-field (0 = pinhole).
    pub fn aperture(mut self, radius: f64) -> Self {
        self.desc.aperture = radius.max(0.0);
        self
    }

    /// Orbit around `center` at the given `distance`, `elevation` (radians)
    /// and `azimuth` (radians).
    pub fn orbit(mut self, center: [f64; 3], distance: f64, elevation: f64, azimuth: f64) -> Self {
        let dist = distance.max(0.5);
        let eye_x = center[0] + dist * elevation.cos() * azimuth.cos();
        let eye_y = center[1] + dist * elevation.sin();
        let eye_z = center[2] + dist * elevation.cos() * azimuth.sin();
        self.desc.eye = [eye_x, eye_y, eye_z];
        self.desc.target = center;
        self
    }

    /// Auto-frame a set of sphere centres and radii so all objects are visible.
    pub fn auto_frame(mut self, objects: &[([f64; 3], f64)]) -> Self {
        if objects.is_empty() {
            return self;
        }
        let n = objects.len() as f64;
        let cx: f64 = objects.iter().map(|(p, _)| p[0]).sum::<f64>() / n;
        let cy: f64 = objects.iter().map(|(p, _)| p[1]).sum::<f64>() / n;
        let cz: f64 = objects.iter().map(|(p, _)| p[2]).sum::<f64>() / n;
        let extent = objects
            .iter()
            .map(|(p, r)| {
                let dx = p[0] - cx;
                let dy = p[1] - cy;
                let dz = p[2] - cz;
                (dx * dx + dy * dy + dz * dz).sqrt() + r
            })
            .fold(1.0_f64, f64::max);
        let dist = extent * 2.8;
        self.desc.eye = [cx + dist * 0.7, cy + dist * 0.45, cz + dist * 0.7];
        self.desc.target = [cx, cy, cz];
        self
    }

    /// Return the descriptor (for passing to [`SceneBuilder::with_camera`]).
    pub fn descriptor(&self) -> CameraDesc {
        self.desc
    }

    /// Build the low-level engine [`Camera`] for the given aspect ratio.
    pub fn build(&self, aspect_ratio: f64) -> Camera {
        let eye = Vec3::new(self.desc.eye[0], self.desc.eye[1], self.desc.eye[2]);
        let target = Vec3::new(self.desc.target[0], self.desc.target[1], self.desc.target[2]);
        let mut cam = Camera::look_at(
            eye,
            target,
            Vec3::new(0.0, 1.0, 0.0),
            self.desc.fov_degrees,
            aspect_ratio,
        );
        if self.desc.aperture > 0.0 {
            cam = cam.with_physical_lens(self.desc.aperture, 0.0, Vec3::ZERO);
        }
        cam
    }
}
