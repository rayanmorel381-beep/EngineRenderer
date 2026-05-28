use crate::api::types::CameraDesc;
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

#[derive(Debug, Clone, Copy, Default)]
/// Fluent camera builder used by API consumers.
pub struct CameraController {
    desc: CameraDesc,
}

impl CameraController {
    /// Creates a default camera controller.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets camera eye and target positions.
    pub fn look_at(mut self, eye: [f64; 3], target: [f64; 3]) -> Self {
        self.desc.eye = eye;
        self.desc.target = target;
        self
    }

    /// Sets vertical field-of-view in degrees.
    pub fn fov(mut self, degrees: f64) -> Self {
        self.desc.fov_degrees = degrees.clamp(10.0, 120.0);
        self
    }

    /// Sets lens aperture radius.
    pub fn aperture(mut self, radius: f64) -> Self {
        self.desc.aperture = radius.max(0.0);
        self
    }

    /// Places the camera on an orbit around a center point.
    pub fn orbit(mut self, center: [f64; 3], distance: f64, elevation: f64, azimuth: f64) -> Self {
        let dist = distance.max(0.5);
        let eye_x = center[0] + dist * elevation.cos() * azimuth.cos();
        let eye_y = center[1] + dist * elevation.sin();
        let eye_z = center[2] + dist * elevation.cos() * azimuth.sin();
        self.desc.eye = [eye_x, eye_y, eye_z];
        self.desc.target = center;
        self
    }

    /// Computes an automatic framing from object positions and radii.
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

    /// Returns the current camera descriptor.
    pub fn descriptor(&self) -> CameraDesc {
        self.desc
    }

    /// Builds a ray-tracing camera for the given aspect ratio.
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
