
use crate::core::engine::rendering::raytracing::Vec3;

use super::frustum_corners::FrustumCorners;

#[derive(Debug, Clone, Copy)]
pub struct LightMatrix {
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    pub origin: Vec3,
    pub half_extent: f64,
    pub near_z: f64,
    pub far_z: f64,
}

impl LightMatrix {
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

    pub fn project(&self, point: Vec3) -> Vec3 {
        let local = point - self.origin;
        let x = local.dot(self.right) / self.half_extent.max(f64::EPSILON);
        let y = local.dot(self.up) / self.half_extent.max(f64::EPSILON);
        let z = local.dot(self.forward);
        let ndc_z = (z - self.near_z) / (self.far_z - self.near_z).max(f64::EPSILON);
        Vec3::new(x, y, ndc_z)
    }

    #[inline]
    pub fn is_in_bounds(&self, projected: Vec3) -> bool {
        projected.x.abs() <= 1.0
            && projected.y.abs() <= 1.0
            && projected.z >= 0.0
            && projected.z <= 1.0
    }
}
