//! Cascaded Shadow Map (CSM) split computation, lookup, and blending.
//!
//! [`ShadowCascade`] holds per-split configuration and optional
//! light-space matrices.  Split boundaries use the classic
//! logarithmic / linear blend controlled by a `lambda` parameter.

use crate::core::engine::rendering::raytracing::{Camera, Scene};
use crate::core::engine::rendering::utils::{lerp, saturate};

use super::frustum_corners::FrustumCorners;
use super::light_matrix::LightMatrix;

// ── Per-cascade configuration ───────────────────────────────────────────

/// Parameters for a single shadow cascade (split range, resolution, bias).
#[derive(Debug, Clone, Copy)]
pub struct CascadeConfig {
    /// Near depth of this cascade's split.
    pub split_near: f64,
    /// Far depth of this cascade's split.
    pub split_far: f64,
    /// Shadow-map resolution for this cascade (e.g. 2048, 1024…).
    pub resolution: u32,
    /// Constant depth bias to prevent self-shadowing (shadow acne).
    pub bias: f64,
    /// Normal-direction bias — offsets the sample along the surface normal.
    pub normal_bias: f64,
}

// ── Shadow cascade set ──────────────────────────────────────────────────

/// A complete set of cascaded shadow maps.
///
/// [`build_with_camera`](Self::build_with_camera) computes split ranges
/// and fits one [`LightMatrix`] per cascade from the active camera.
#[derive(Debug, Clone)]
pub struct ShadowCascade {
    /// Per-cascade split ranges and quality settings.
    pub cascades: Vec<CascadeConfig>,
    /// Orthographic projection per cascade (empty when built without a camera).
    pub light_matrices: Vec<LightMatrix>,
    /// Heuristic estimate of how much occlusion the scene produces `[0, 1]`.
    pub occlusion_estimate: f64,
    /// Global shadow strength multiplier.
    pub shadow_strength: f64,
    /// Width (as a fraction of the cascade's far depth) over which two
    /// adjacent cascades are blended to avoid hard seams.
    pub cascade_blend_width: f64,
}

impl ShadowCascade {
    /// Full CSM construction: computes split ranges **and** fits a
    /// [`LightMatrix`] for each cascade using the camera frustum.
    pub fn build_with_camera(
        scene: &Scene,
        camera: &Camera,
        near: f64,
        far: f64,
        num_cascades: usize,
    ) -> Self {
        let occluder_weight: f64 = scene
            .objects
            .iter()
            .map(|obj| {
                let dist = (obj.center - camera.origin).length();
                obj.radius / (1.0 + dist)
            })
            .sum();

        let occlusion_estimate = (occluder_weight / 6.0).clamp(0.0, 0.65);
        let cascades = Self::compute_cascade_splits(near, far, num_cascades, 0.85);

        let light_dir = scene.sun.direction.normalize();
        let fov = 60.0_f64.to_radians();
        let aspect = 1.0;

        let light_matrices = cascades
            .iter()
            .map(|c| {
                let frustum =
                    FrustumCorners::from_camera(camera, fov, aspect, c.split_near, c.split_far);
                LightMatrix::from_direction_and_frustum(light_dir, &frustum)
            })
            .collect();

        Self {
            light_matrices,
            cascades,
            occlusion_estimate,
            shadow_strength: 0.85,
            cascade_blend_width: 0.05,
        }
    }

    /// Computes cascade split boundaries using a logarithmic / linear
    /// blend, governed by `lambda` (`0.0` = pure linear, `1.0` = pure
    /// logarithmic).
    fn compute_cascade_splits(
        near: f64,
        far: f64,
        count: usize,
        lambda: f64,
    ) -> Vec<CascadeConfig> {
        let count = count.clamp(1, 6);
        let mut splits = Vec::with_capacity(count);

        for i in 0..count {
            let t0 = i as f64 / count as f64;
            let t1 = (i + 1) as f64 / count as f64;

            let log_near = near * (far / near).powf(t0);
            let log_far = near * (far / near).powf(t1);
            let lin_near = near + (far - near) * t0;
            let lin_far = near + (far - near) * t1;

            let split_near = lerp(lin_near, log_near, lambda);
            let split_far = lerp(lin_far, log_far, lambda);

            let resolution = match i {
                0 => 2048,
                1 => 1024,
                2 => 512,
                _ => 256,
            };

            splits.push(CascadeConfig {
                split_near,
                split_far,
                resolution,
                bias: 0.0005 * (i as f64 + 1.0),
                normal_bias: 0.002 * (i as f64 + 1.0),
            });
        }

        splits
    }

    /// Returns the cascade index that contains `depth`, or the last
    /// cascade when `depth` exceeds all splits.
    pub fn cascade_index_for_depth(&self, depth: f64) -> usize {
        for (i, cascade) in self.cascades.iter().enumerate() {
            if depth < cascade.split_far {
                return i;
            }
        }
        self.cascades.len().saturating_sub(1)
    }

    /// Returns a `[0, 1]` blend factor indicating how close `depth` is
    /// to the far edge of its cascade.
    ///
    /// `0.0` → firmly inside the cascade; `1.0` → right at the boundary
    /// (should sample the next cascade and blend).
    pub fn cascade_blend_factor(&self, depth: f64, cascade_idx: usize) -> f64 {
        if cascade_idx >= self.cascades.len() {
            return 0.0;
        }
        let far = self.cascades[cascade_idx].split_far;
        let blend_start = far * (1.0 - self.cascade_blend_width);
        if depth < blend_start {
            0.0
        } else {
            saturate((depth - blend_start) / (far - blend_start).max(f64::EPSILON))
        }
    }
}
