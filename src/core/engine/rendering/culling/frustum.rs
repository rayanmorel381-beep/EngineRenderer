
use crate::core::engine::rendering::raytracing::{Camera, Vec3};

// ── Half-space plane ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f64,
}

impl Plane {
    pub fn new(normal: Vec3, point: Vec3) -> Self {
        let n = normal.normalize();
        Self {
            normal: n,
            distance: n.dot(point),
        }
    }

    #[inline]
    pub fn signed_distance(&self, point: Vec3) -> f64 {
        self.normal.dot(point) - self.distance
    }
}

// ── Cull result ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullResult {
    Inside,
    Intersecting,
    Outside,
}

// ── View frustum ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn from_camera(camera: &Camera, fov_rad: f64, aspect: f64, near: f64, far: f64) -> Self {
        let forward = camera.direction.normalize();
        let world_up = if forward.y.abs() > 0.999 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();

        let half_v = (fov_rad * 0.5).tan();
        let half_h = half_v * aspect;

        let near_center = camera.origin + forward * near;
        let far_center = camera.origin + forward * far;

        // Near / far planes
        let near_plane = Plane::new(forward, near_center);
        let far_plane = Plane::new(-forward, far_center);

        // Side planes — normals point inward
        let left_normal = (forward * half_h + right).cross(up).normalize();
        let right_normal = up.cross(forward * half_h - right).normalize();
        let top_normal = (forward * half_v + up).cross(right).normalize();
        let bottom_normal = right.cross(forward * half_v - up).normalize();

        // Flip any normal that doesn't face the frustum interior
        let center = (near_center + far_center) / 2.0;
        let left_plane = Self::orient_plane(left_normal, camera.origin, center);
        let right_plane = Self::orient_plane(right_normal, camera.origin, center);
        let top_plane = Self::orient_plane(top_normal, camera.origin, center);
        let bottom_plane = Self::orient_plane(bottom_normal, camera.origin, center);

        Self {
            planes: [near_plane, far_plane, left_plane, right_plane, top_plane, bottom_plane],
        }
    }

    fn orient_plane(normal: Vec3, plane_point: Vec3, interior_point: Vec3) -> Plane {
        let plane = Plane::new(normal, plane_point);
        if plane.signed_distance(interior_point) < 0.0 {
            Plane::new(-normal, plane_point)
        } else {
            plane
        }
    }

    pub fn contains_sphere(&self, center: Vec3, radius: f64) -> CullResult {
        let mut all_inside = true;
        for plane in &self.planes {
            let dist = plane.signed_distance(center);
            if dist < -radius {
                return CullResult::Outside;
            }
            if dist < radius {
                all_inside = false;
            }
        }
        if all_inside {
            CullResult::Inside
        } else {
            CullResult::Intersecting
        }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.signed_distance(point) >= 0.0)
    }

    pub fn contains_aabb(&self, aabb_min: Vec3, aabb_max: Vec3) -> CullResult {
        let mut all_inside = true;
        for plane in &self.planes {
            // Positive vertex — furthest in the direction of the plane normal
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { aabb_max.x } else { aabb_min.x },
                if plane.normal.y >= 0.0 { aabb_max.y } else { aabb_min.y },
                if plane.normal.z >= 0.0 { aabb_max.z } else { aabb_min.z },
            );
            // Negative vertex — closest along the normal
            let n = Vec3::new(
                if plane.normal.x >= 0.0 { aabb_min.x } else { aabb_max.x },
                if plane.normal.y >= 0.0 { aabb_min.y } else { aabb_max.y },
                if plane.normal.z >= 0.0 { aabb_min.z } else { aabb_max.z },
            );

            if plane.signed_distance(p) < 0.0 {
                return CullResult::Outside;
            }
            if plane.signed_distance(n) < 0.0 {
                all_inside = false;
            }
        }
        if all_inside {
            CullResult::Inside
        } else {
            CullResult::Intersecting
        }
    }
}
