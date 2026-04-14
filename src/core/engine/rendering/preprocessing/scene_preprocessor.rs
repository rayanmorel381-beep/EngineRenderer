use crate::core::engine::rendering::raytracing::{Camera, Scene, Vec3};

#[derive(Debug, Clone)]
pub struct SceneAnalysis {
    pub total_primitives: usize,
    pub scene_bounds_min: Vec3,
    pub scene_bounds_max: Vec3,
    pub scene_radius: f64,
    pub scene_center: Vec3,
    pub dominant_light_direction: Vec3,
    pub average_object_radius: f64,
    pub max_object_radius: f64,
    pub emissive_count: usize,
    pub transmissive_count: usize,
    pub high_detail_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct PreprocessedCamera {
    pub near_plane: f64,
    pub far_plane: f64,
    pub frustum_half_width: f64,
    pub frustum_half_height: f64,
}

#[derive(Debug, Clone)]
pub struct PreprocessedScene {
    pub analysis: SceneAnalysis,
    pub camera_info: PreprocessedCamera,
    pub sorted_object_indices: Vec<usize>,
    pub sorted_triangle_indices: Vec<usize>,
}

pub struct ScenePreprocessor;

impl ScenePreprocessor {
    pub fn analyze(scene: &Scene, camera: &Camera) -> PreprocessedScene {
        let analysis = Self::analyze_scene(scene);
        let camera_info = Self::analyze_camera(camera, &analysis);
        let sorted_object_indices = Self::sort_objects_by_distance(scene, camera);
        let sorted_triangle_indices = Self::sort_triangles_by_distance(scene, camera);

        PreprocessedScene {
            analysis,
            camera_info,
            sorted_object_indices,
            sorted_triangle_indices,
        }
    }

    fn analyze_scene(scene: &Scene) -> SceneAnalysis {
        let total_primitives = scene.objects.len() + scene.triangles.len();

        let mut bounds_min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut bounds_max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        let mut center_sum = Vec3::ZERO;
        let mut radius_sum = 0.0;
        let mut max_radius = 0.0_f64;
        let mut emissive_count = 0;
        let mut transmissive_count = 0;
        let mut high_detail_count = 0;

        for obj in &scene.objects {
            let rmin = obj.center - Vec3::splat(obj.radius);
            let rmax = obj.center + Vec3::splat(obj.radius);
            bounds_min = Vec3::new(
                bounds_min.x.min(rmin.x),
                bounds_min.y.min(rmin.y),
                bounds_min.z.min(rmin.z),
            );
            bounds_max = Vec3::new(
                bounds_max.x.max(rmax.x),
                bounds_max.y.max(rmax.y),
                bounds_max.z.max(rmax.z),
            );
            center_sum += obj.center;
            radius_sum += obj.radius;
            max_radius = max_radius.max(obj.radius);

            if obj.material.emission.length_squared() > 0.01 {
                emissive_count += 1;
            }
            if obj.material.transmission > 0.05 {
                transmissive_count += 1;
            }
            if obj.material.clearcoat > 0.15
                || obj.material.iridescence > 0.15
                || obj.material.anisotropy > 0.15
            {
                high_detail_count += 1;
            }
        }

        for tri in &scene.triangles {
            for point in [tri.a, tri.b, tri.c] {
                bounds_min = Vec3::new(
                    bounds_min.x.min(point.x),
                    bounds_min.y.min(point.y),
                    bounds_min.z.min(point.z),
                );
                bounds_max = Vec3::new(
                    bounds_max.x.max(point.x),
                    bounds_max.y.max(point.y),
                    bounds_max.z.max(point.z),
                );
            }
            let centroid = (tri.a + tri.b + tri.c) / 3.0;
            center_sum += centroid;
        }

        let count = total_primitives.max(1) as f64;
        let scene_center = center_sum / count;
        let scene_radius = (bounds_max - bounds_min).length() * 0.5;
        let average_object_radius = if scene.objects.is_empty() {
            0.0
        } else {
            radius_sum / scene.objects.len() as f64
        };

        SceneAnalysis {
            total_primitives,
            scene_bounds_min: bounds_min,
            scene_bounds_max: bounds_max,
            scene_radius,
            scene_center,
            dominant_light_direction: (-scene.sun.direction).normalize(),
            average_object_radius,
            max_object_radius: max_radius,
            emissive_count,
            transmissive_count,
            high_detail_count,
        }
    }

    fn analyze_camera(camera: &Camera, analysis: &SceneAnalysis) -> PreprocessedCamera {
        let dist_to_center = (camera.origin - analysis.scene_center).length();
        let near_plane = (dist_to_center - analysis.scene_radius).max(0.01);
        let far_plane = dist_to_center + analysis.scene_radius + analysis.max_object_radius * 2.0;

        PreprocessedCamera {
            near_plane,
            far_plane,
            frustum_half_width: analysis.scene_radius * 1.2,
            frustum_half_height: analysis.scene_radius * 1.2,
        }
    }

    fn sort_objects_by_distance(scene: &Scene, camera: &Camera) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..scene.objects.len()).collect();
        indices.sort_by(|&a, &b| {
            let da = (scene.objects[a].center - camera.origin).length_squared();
            let db = (scene.objects[b].center - camera.origin).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    fn sort_triangles_by_distance(scene: &Scene, camera: &Camera) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..scene.triangles.len()).collect();
        indices.sort_by(|&a, &b| {
            let ca = (scene.triangles[a].a + scene.triangles[a].b + scene.triangles[a].c) / 3.0;
            let cb = (scene.triangles[b].a + scene.triangles[b].b + scene.triangles[b].c) / 3.0;
            let da = (ca - camera.origin).length_squared();
            let db = (cb - camera.origin).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AdaptiveQualitySettings {
    pub sample_multiplier: f64,
    pub bounce_limit: u32,
    pub shadow_quality: f64,
    pub ao_quality: f64,
    pub volumetric_quality: f64,
}

impl AdaptiveQualitySettings {
    pub fn from_analysis(analysis: &SceneAnalysis, target_ms: f64) -> Self {
        let complexity = analysis.total_primitives as f64
            + analysis.emissive_count as f64 * 2.0
            + analysis.transmissive_count as f64 * 4.0
            + analysis.high_detail_count as f64 * 1.5;

        let normalized_budget = (target_ms / 16.0).max(1.0);
        let pressure = complexity / (normalized_budget * 4800.0);
        let budget_factor = (1.02 - pressure * 0.08).clamp(0.55, 1.10);

        Self {
            sample_multiplier: budget_factor.clamp(0.82, 1.10),
            bounce_limit: if budget_factor > 0.85 { 4 } else if budget_factor > 0.60 { 3 } else { 2 },
            shadow_quality: budget_factor.clamp(0.55, 1.0),
            ao_quality: budget_factor.clamp(0.50, 1.0),
            volumetric_quality: (budget_factor * 0.90).clamp(0.45, 1.0),
        }
    }
}
