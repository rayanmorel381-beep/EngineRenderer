//! High-level scene culler that combines distance, contribution,
//! back-face, and frustum tests into a single configurable pass.
//!
//! # Example
//! ```ignore
//! let culler = SceneCuller::new(500.0)
//!     .with_screen_params(fov, 1080.0)
//!     .with_contribution_threshold(1.0)
//!     .with_backface_culling(true);
//!
//! let (visible, stats) = culler.cull_scene_with_stats(&scene, &camera);
//! eprintln!("culled {:.0}% spheres", stats.sphere_ratio() * 100.0);
//! ```

use crate::core::engine::rendering::raytracing::{Camera, Scene};

use super::frustum::{CullResult, Frustum};
use super::helpers::{is_backfacing, is_contribution_negligible, triangle_normal};

// ── Culling statistics ──────────────────────────────────────────────────

/// Lightweight counters filled during a cull pass — useful for profiling
/// and adaptive quality tuning.
#[derive(Debug, Clone, Copy)]
pub struct CullStats {
    /// Number of spheres before culling.
    pub total_spheres: usize,
    /// Number of triangles before culling.
    pub total_triangles: usize,
    /// Spheres removed by the cull pass.
    pub culled_spheres: usize,
    /// Triangles removed by the cull pass.
    pub culled_triangles: usize,
}

impl CullStats {
    /// Fraction of spheres that were culled (`0.0` = none, `1.0` = all).
    pub fn sphere_ratio(&self) -> f64 {
        if self.total_spheres == 0 {
            0.0
        } else {
            self.culled_spheres as f64 / self.total_spheres as f64
        }
    }

    /// Fraction of triangles that were culled (`0.0` = none, `1.0` = all).
    pub fn triangle_ratio(&self) -> f64 {
        if self.total_triangles == 0 {
            0.0
        } else {
            self.culled_triangles as f64 / self.total_triangles as f64
        }
    }
}

// ── Scene culler ────────────────────────────────────────────────────────

/// Configurable multi-strategy scene culler.
///
/// Combines:
/// * **Distance culling** — objects beyond `max_distance + distance_margin`
///   are rejected.
/// * **Contribution culling** — objects whose screen projection is smaller
///   than `min_screen_pixels` are rejected.
/// * **Back-face culling** — triangles facing away from the camera are
///   rejected (optional).
///
/// Use [`cull_with_frustum`](SceneCuller::cull_with_frustum) to add a
/// frustum pass on top.
#[derive(Debug, Clone, Copy)]
pub struct SceneCuller {
    /// Hard cut-off distance for spheres/triangles.
    max_distance: f64,
    /// Extra margin added to `max_distance` to soften the boundary.
    distance_margin: f64,
    /// Minimum screen-space pixel diameter to keep an object.
    min_screen_pixels: f64,
    /// Vertical FOV in radians (used for contribution culling).
    fov_rad: f64,
    /// Render target height in pixels.
    screen_height: f64,
    /// Whether to discard back-facing triangles.
    backface_cull: bool,
}

impl SceneCuller {
    /// Creates a culler with a maximum draw distance and sensible defaults.
    pub fn new(max_distance: f64) -> Self {
        Self {
            max_distance,
            distance_margin: 6.0,
            min_screen_pixels: 0.5,
            fov_rad: 60.0_f64.to_radians(),
            screen_height: 720.0,
            backface_cull: true,
        }
    }

    /// Sets the screen parameters used for contribution culling.
    pub fn with_screen_params(mut self, fov_rad: f64, screen_height: f64) -> Self {
        self.fov_rad = fov_rad;
        self.screen_height = screen_height;
        self
    }

    /// Minimum projected pixel-size below which objects are discarded.
    pub fn with_contribution_threshold(mut self, min_pixels: f64) -> Self {
        self.min_screen_pixels = min_pixels;
        self
    }

    /// Toggles triangle back-face culling.
    pub fn with_backface_culling(mut self, enabled: bool) -> Self {
        self.backface_cull = enabled;
        self
    }

    /// Returns a new `Scene` containing only the objects and triangles
    /// that pass distance + contribution + back-face tests.
    pub fn cull_scene(&self, scene: &Scene, camera: &Camera) -> Scene {
        let cull_limit = self.max_distance + self.distance_margin;

        let objects = scene
            .objects
            .iter()
            .copied()
            .filter(|object| {
                let distance = (object.center - camera.origin).length() - object.radius;
                if distance > cull_limit {
                    return false;
                }
                if distance > 0.0
                    && is_contribution_negligible(
                        object.radius,
                        distance,
                        self.fov_rad,
                        self.screen_height,
                        self.min_screen_pixels,
                    )
                {
                    return false;
                }
                true
            })
            .collect();

        let view_dir = camera.direction.normalize();
        let triangles = scene
            .triangles
            .iter()
            .copied()
            .filter(|tri| {
                let centroid = (tri.a + tri.b + tri.c) / 3.0;
                let dist = (centroid - camera.origin).length();
                if dist > cull_limit {
                    return false;
                }
                // View-direction contribution: reject triangles far off the view axis
                let to_tri = (centroid - camera.origin).normalize();
                let view_dot = view_dir.dot(to_tri);
                if view_dot < -0.15 && dist > self.max_distance * 0.5 {
                    return false;
                }
                if self.backface_cull {
                    let normal = triangle_normal(tri.a, tri.b, tri.c);
                    if is_backfacing(normal, to_tri) {
                        return false;
                    }
                }
                true
            })
            .collect();

        Scene {
            objects,
            triangles,
            sun: scene.sun,
            area_lights: scene.area_lights.clone(),
            sky_top: scene.sky_top,
            sky_bottom: scene.sky_bottom,
            exposure: scene.exposure,
            volume: scene.volume,
            hdri: scene.hdri.clone(),
            solar_elevation: scene.solar_elevation,
        }
    }

    /// Same as [`cull_scene`](Self::cull_scene) but also returns
    /// [`CullStats`] profiling counters.
    pub fn cull_scene_with_stats(&self, scene: &Scene, camera: &Camera) -> (Scene, CullStats) {
        let total_spheres = scene.objects.len();
        let total_triangles = scene.triangles.len();
        let culled = self.cull_scene(scene, camera);
        let stats = CullStats {
            total_spheres,
            total_triangles,
            culled_spheres: total_spheres - culled.objects.len(),
            culled_triangles: total_triangles - culled.triangles.len(),
        };
        (culled, stats)
    }

    /// Culls using an explicit [`Frustum`] instead of distance heuristics.
    ///
    /// Triangles are bounded by a sphere around their centroid for the
    /// containment test.
    pub fn cull_with_frustum(&self, scene: &Scene, frustum: &Frustum) -> Scene {
        let objects = scene
            .objects
            .iter()
            .copied()
            .filter(|obj| {
                frustum.contains_sphere(obj.center, obj.radius) != CullResult::Outside
            })
            .collect();

        let triangles = scene
            .triangles
            .iter()
            .copied()
            .filter(|tri| {
                let centroid = (tri.a + tri.b + tri.c) / 3.0;
                let half = ((tri.a - centroid).length())
                    .max((tri.b - centroid).length())
                    .max((tri.c - centroid).length());
                frustum.contains_sphere(centroid, half) != CullResult::Outside
            })
            .collect();

        Scene {
            objects,
            triangles,
            sun: scene.sun,
            area_lights: scene.area_lights.clone(),
            sky_top: scene.sky_top,
            sky_bottom: scene.sky_bottom,
            exposure: scene.exposure,
            volume: scene.volume,
            hdri: scene.hdri.clone(),
            solar_elevation: scene.solar_elevation,
        }
    }
}
