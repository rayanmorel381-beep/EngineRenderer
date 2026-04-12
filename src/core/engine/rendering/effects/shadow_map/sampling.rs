//! Shadow sampling routines: Percentage-Closer Filtering (PCF) and
//! screen-space contact shadows.
//!
//! These functions operate on the live scene geometry via ray casts —
//! no pre-baked shadow map texture is required.

use crate::core::engine::rendering::raytracing::{Ray, Scene, Vec3};

// ── PCF shadow sampling ─────────────────────────────────────────────────

/// Evaluates soft shadow visibility at `hit_point` using a
/// Percentage-Closer Filter (PCF) with a square `kernel_size × kernel_size`
/// grid of jittered rays.
///
/// Returns a visibility factor in `[0, 1]`: `0.0` = fully in shadow,
/// `1.0` = fully lit.
///
/// # Bias
/// * `bias`        – constant offset along `light_dir` to avoid shadow acne.
/// * `normal_bias` – offset along the surface `normal`.
pub fn pcf_shadow(
    scene: &Scene,
    hit_point: Vec3,
    normal: Vec3,
    light_dir: Vec3,
    bias: f64,
    normal_bias: f64,
    kernel_size: u32,
) -> f64 {
    let biased_origin = hit_point + normal * normal_bias + light_dir * bias;
    let samples = kernel_size.max(1);

    // Single-sample fast path.
    if samples == 1 {
        let ray = Ray::new(biased_origin, light_dir);
        return if scene.is_occluded(&ray, 1e6) {
            0.0
        } else {
            1.0
        };
    }

    let (tangent, bitangent) = crate::core::engine::rendering::utils::build_tangent_frame(light_dir);
    let spread = 0.008;
    let mut lit = 0.0;
    let total = samples * samples;

    for iy in 0..samples {
        for ix in 0..samples {
            let fx = (ix as f64 / (samples - 1).max(1) as f64 - 0.5) * spread;
            let fy = (iy as f64 / (samples - 1).max(1) as f64 - 0.5) * spread;
            let jittered = (light_dir + tangent * fx + bitangent * fy).normalize();
            let ray = Ray::new(biased_origin, jittered);
            if !scene.is_occluded(&ray, 1e6) {
                lit += 1.0;
            }
        }
    }

    lit / total as f64
}

// ── Contact shadows (PCSS-like) ─────────────────────────────────────────

/// Ray-marches from `hit_point` toward the light to detect short-range
/// self-contact shadows.
///
/// Returns a visibility factor in `[0, 1]`:
/// * `1.0` — no contact shadow detected.
/// * `0.0 …` — occluded; the value fades linearly with the step at
///   which the first occlusion was found (farther → lighter shadow).
///
/// `max_distance` controls how far the march extends;
/// `steps` controls the quality / cost trade-off.
pub fn contact_shadow(
    scene: &Scene,
    hit_point: Vec3,
    normal: Vec3,
    to_light: Vec3,
    max_distance: f64,
    steps: u32,
) -> f64 {
    let biased = hit_point + normal * 0.005;
    let step_size = max_distance / steps.max(1) as f64;

    for i in 0..steps {
        let t = (i as f64 + 0.5) * step_size;
        let sample = biased + to_light * t;
        let ray = Ray::new(sample, to_light);
        if scene.is_occluded(&ray, step_size * 1.5) {
            let fade = (i as f64 + 1.0) / steps as f64;
            return 1.0 - fade;
        }
    }

    1.0
}
