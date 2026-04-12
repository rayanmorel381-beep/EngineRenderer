//! Camera frustum corner extraction for cascade fitting.
//!
//! [`FrustumCorners`] stores the eight vertices of a truncated view
//! pyramid.  It is used by the cascade builder to compute tight
//! light-space bounding volumes.

use crate::core::engine::rendering::raytracing::{Camera, Vec3};

/// The eight corners of a camera frustum sub-volume (near quad + far quad).
///
/// Winding order per quad: **top-left, top-right, bottom-right, bottom-left**
/// (when viewed from the camera's perspective).
#[derive(Debug, Clone, Copy)]
pub struct FrustumCorners {
    /// Four corners on the near clip plane.
    pub near: [Vec3; 4],
    /// Four corners on the far clip plane.
    pub far: [Vec3; 4],
}

impl FrustumCorners {
    /// Returns the centroid of all eight corners.
    pub fn center(&self) -> Vec3 {
        let mut sum = Vec3::ZERO;
        for p in &self.near {
            sum += *p;
        }
        for p in &self.far {
            sum += *p;
        }
        sum / 8.0
    }

    /// Returns the smallest sphere radius enclosing all eight corners
    /// when centred on [`center`](Self::center).
    pub fn bounding_radius(&self) -> f64 {
        let c = self.center();
        let mut max_r2 = 0.0_f64;
        for p in self.near.iter().chain(self.far.iter()) {
            max_r2 = max_r2.max((*p - c).length_squared());
        }
        max_r2.sqrt()
    }

    /// Extracts frustum corners from a camera, FOV, aspect ratio and
    /// `[near … far]` depth range.
    ///
    /// This is the standard decomposition used by CSM (Cascaded Shadow
    /// Maps) to fit tight ortho projections for each split.
    pub fn from_camera(
        camera: &Camera,
        fov: f64,
        aspect: f64,
        near: f64,
        far: f64,
    ) -> Self {
        let forward = camera.direction.normalize();
        let world_up = if forward.y.abs() > 0.999 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();

        let half_v = (fov * 0.5).tan();
        let half_h = half_v * aspect;

        let near_center = camera.origin + forward * near;
        let far_center = camera.origin + forward * far;

        let nh = half_h * near;
        let nv = half_v * near;
        let fh = half_h * far;
        let fv = half_v * far;

        Self {
            near: [
                near_center + up * nv - right * nh,
                near_center + up * nv + right * nh,
                near_center - up * nv + right * nh,
                near_center - up * nv - right * nh,
            ],
            far: [
                far_center + up * fv - right * fh,
                far_center + up * fv + right * fh,
                far_center - up * fv + right * fh,
                far_center - up * fv - right * fh,
            ],
        }
    }
}
