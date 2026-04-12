//! Orthographic light-space projection matrix for shadow mapping.
//!
//! [`LightMatrix`] wraps the axes and extent of an orthographic volume
//! aligned to a directional light, sized to enclose a given set of
//! [`FrustumCorners`](super::FrustumCorners).

use crate::core::engine::rendering::raytracing::Vec3;

use super::frustum_corners::FrustumCorners;

/// An orthographic projection aligned along a directional light.
///
/// Projects world-space points into a `[-1, 1]² × [0, 1]` NDC cube that
/// represents the shadow map.
#[derive(Debug, Clone, Copy)]
pub struct LightMatrix {
    /// Local right axis (perpendicular to light direction).
    pub right: Vec3,
    /// Local up axis (perpendicular to light direction and right).
    pub up: Vec3,
    /// Light propagation direction (normalised).
    pub forward: Vec3,
    /// Origin of the light-space volume (pulled back from the frustum).
    pub origin: Vec3,
    /// Half-width of the ortho box in world units.
    pub half_extent: f64,
    /// Near depth of the ortho box (usually `0.0`).
    pub near_z: f64,
    /// Far depth of the ortho box.
    pub far_z: f64,
}

impl LightMatrix {
    /// Builds a tight orthographic volume from a light direction and a set
    /// of frustum corners.
    ///
    /// The volume is centred on the frustum and sized to its bounding
    /// sphere, then pulled back by `2×radius` along the light direction
    /// to avoid near-plane clipping.
    pub fn from_direction_and_frustum(light_dir: Vec3, frustum: &FrustumCorners) -> Self {
        let forward = light_dir.normalize();
        let world_up = if forward.y.abs() > 0.999 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let right = world_up.cross(forward).normalize();
        let up = forward.cross(right).normalize();

        let center = frustum.center();
        let radius = frustum.bounding_radius();

        Self {
            right,
            up,
            forward,
            origin: center - forward * radius * 2.0,
            half_extent: radius,
            near_z: 0.0,
            far_z: radius * 4.0,
        }
    }

    /// Projects a world-space point into light-space NDC.
    ///
    /// Returns `(x, y, z)` where `x, y ∈ [-1, 1]` and `z ∈ [0, 1]`.
    pub fn project(&self, point: Vec3) -> Vec3 {
        let local = point - self.origin;
        let x = local.dot(self.right) / self.half_extent.max(f64::EPSILON);
        let y = local.dot(self.up) / self.half_extent.max(f64::EPSILON);
        let z = local.dot(self.forward);
        let ndc_z = (z - self.near_z) / (self.far_z - self.near_z).max(f64::EPSILON);
        Vec3::new(x, y, ndc_z)
    }

    /// Returns `true` when a projected point lies inside the shadow-map
    /// volume.
    #[inline]
    pub fn is_in_bounds(&self, projected: Vec3) -> bool {
        projected.x.abs() <= 1.0
            && projected.y.abs() <= 1.0
            && projected.z >= 0.0
            && projected.z <= 1.0
    }
}
