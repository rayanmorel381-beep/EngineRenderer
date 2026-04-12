//! Lightweight culling helpers: contribution, back-face, and occlusion tests.
//!
//! These free-standing functions complement the heavier [`SceneCuller`] and
//! [`Frustum`] by providing per-object or per-triangle acceptance tests that
//! can be composed into custom pipelines.

use crate::core::engine::rendering::raytracing::{Sphere, Vec3};

// ── Contribution culling ────────────────────────────────────────────────

/// Estimates how many screen-pixels an object of `object_radius` subtends
/// when seen from `camera_distance` with the given vertical FOV and
/// screen height.
///
/// The result is an **approximate pixel diameter** — useful as a
/// threshold to decide whether a distant object is worth rendering.
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

/// Returns `true` when an object's projected size on screen is below
/// `pixel_threshold` — i.e. it contributes fewer pixels than the minimum
/// and can be safely culled.
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

/// Returns `true` when the triangle faces **away** from the viewer.
///
/// `triangle_normal` must be the face normal (world-space) and
/// `view_direction` points **from the camera toward the triangle**.
#[inline]
pub fn is_backfacing(triangle_normal: Vec3, view_direction: Vec3) -> bool {
    triangle_normal.dot(view_direction) > 0.0
}

/// Computes the (un-normalised) face normal from three triangle vertices
/// via the cross-product of two edges.
#[inline]
pub fn triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - a).normalize()
}

// ── Occlusion culling (coarse, sphere-based) ────────────────────────────

/// Cheap, conservative occlusion test using bounding spheres.
///
/// Returns `true` when `occluder` fully hides the target sphere (given
/// by `target_center` / `target_radius`) as seen from `camera_pos`.
///
/// The check compares angular extents: the occluder must be nearer to
/// the camera **and** subtend a larger solid angle than the target,
/// including the angular gap between them.
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
