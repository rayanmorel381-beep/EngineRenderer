//! Frustum geometry primitives for view-volume culling.
//!
//! Provides [`Plane`], [`Frustum`] and [`CullResult`] — the foundational types
//! used by every culling pass (distance, contribution, frustum, occlusion).

use crate::core::engine::rendering::raytracing::{Camera, Vec3};

// ── Half-space plane ────────────────────────────────────────────────────

/// An oriented half-space defined by a unit normal and a signed distance
/// from the origin.
///
/// Points on the positive side (`signed_distance > 0`) are considered
/// *inside* the half-space.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Outward-pointing unit normal.
    pub normal: Vec3,
    /// Signed distance from the world origin along `normal`.
    pub distance: f64,
}

impl Plane {
    /// Constructs a plane from a (possibly non-unit) normal and a point
    /// that lies on the plane.
    ///
    /// The normal is normalised internally; callers do **not** need to
    /// pre-normalise it.
    pub fn new(normal: Vec3, point: Vec3) -> Self {
        let n = normal.normalize();
        Self {
            normal: n,
            distance: n.dot(point),
        }
    }

    /// Returns the signed perpendicular distance from `point` to this plane.
    ///
    /// * **Positive** → the point is on the normal side (inside).
    /// * **Negative** → on the opposite side (outside).
    /// * **Zero**     → exactly on the plane.
    #[inline]
    pub fn signed_distance(&self, point: Vec3) -> f64 {
        self.normal.dot(point) - self.distance
    }
}

// ── Cull result ─────────────────────────────────────────────────────────

/// Result of a containment test against a convex volume (frustum, AABB…).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullResult {
    /// Entirely inside the volume — no clipping needed.
    Inside,
    /// Partially overlapping — may require clipping or conservative accept.
    Intersecting,
    /// Completely outside — safe to discard.
    Outside,
}

// ── View frustum ────────────────────────────────────────────────────────

/// A six-plane convex frustum extracted from camera parameters.
///
/// Plane order: **near, far, left, right, top, bottom**.  All normals
/// point *inward* so that a positive signed-distance means "inside the
/// frustum".
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    /// The six bounding planes `[near, far, left, right, top, bottom]`.
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Builds a perspective frustum from camera pose and lens parameters.
    ///
    /// # Arguments
    /// * `camera`  – camera providing origin + direction.
    /// * `fov_rad` – vertical field-of-view **in radians**.
    /// * `aspect`  – width / height ratio.
    /// * `near`    – near-clip distance (> 0).
    /// * `far`     – far-clip distance  (> near).
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

    /// Ensures the plane normal points toward `interior_point`.
    fn orient_plane(normal: Vec3, plane_point: Vec3, interior_point: Vec3) -> Plane {
        let plane = Plane::new(normal, plane_point);
        if plane.signed_distance(interior_point) < 0.0 {
            Plane::new(-normal, plane_point)
        } else {
            plane
        }
    }

    /// Tests whether a bounding sphere is inside, intersecting, or outside
    /// this frustum.
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

    /// Returns `true` when `point` lies within (or on) every half-space.
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.signed_distance(point) >= 0.0)
    }

    /// Tests an axis-aligned bounding box against the frustum.
    ///
    /// Uses the *P-vertex / N-vertex* method for each plane: the positive
    /// vertex (furthest along the normal) determines early-out; the
    /// negative vertex determines full containment.
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
