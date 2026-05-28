
use crate::core::engine::rendering::raytracing::{Sphere, Vec3};

// ── Contribution culling ────────────────────────────────────────────────

pub fn projected_screen_coverage(
    object_radius: f64,
    camera_distance: f64,
    fov_rad: f64,
    screen_height: f64,
) -> f64 {
    let safe_dist = camera_distance.max(0.001);
    let angular_size = 2.0 * (object_radius / safe_dist).atan();
    angular_size / fov_rad * screen_height
}

pub fn is_contribution_negligible(
    object_radius: f64,
    camera_distance: f64,
    fov_rad: f64,
    screen_height: f64,
    pixel_threshold: f64,
) -> bool {
    projected_screen_coverage(object_radius, camera_distance, fov_rad, screen_height)
        < pixel_threshold
}

// ── Back-face culling ───────────────────────────────────────────────────

#[inline]
pub fn is_backfacing(triangle_normal: Vec3, view_direction: Vec3) -> bool {
    triangle_normal.dot(view_direction) > 0.0
}

#[inline]
pub fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize()
}

// ── Occlusion culling (coarse, sphere-based) ────────────────────────────

pub fn sphere_occludes(
    occluder: &Sphere,
    target_center: Vec3,
    target_radius: f64,
    camera_pos: Vec3,
) -> bool {
    let to_occluder = occluder.center - camera_pos;
    let to_target = target_center - camera_pos;

    let dist_occ = to_occluder.length();
    let dist_tgt = to_target.length();

    // Occluder must be closer than the target.
    if dist_occ >= dist_tgt {
        return false;
    }

    let dir_to_target = to_target / dist_tgt.max(f64::EPSILON);
    let proj = to_occluder.dot(dir_to_target);
    let perp_dist = (to_occluder - dir_to_target * proj).length();

    let angular_occ = (occluder.radius / dist_occ.max(0.001)).atan();
    let angular_tgt = (target_radius / dist_tgt.max(0.001)).atan();
    let angular_sep = (perp_dist / dist_occ.max(0.001)).atan();

    angular_occ > angular_tgt + angular_sep
}
