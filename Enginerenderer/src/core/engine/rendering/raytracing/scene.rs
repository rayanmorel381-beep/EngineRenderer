use super::math::Vec3;
use super::primitives::{HitRecord, Ray, Sphere, Triangle, EPSILON};
use crate::core::engine::rendering::{
    environment::procedural::ProceduralEnvironment,
    effects::volumetric_effects::medium::VolumetricMedium,
};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    /// Light direction.
    pub direction: Vec3,
    /// Light color.
    pub color: Vec3,
    /// Light intensity multiplier.
    pub intensity: f64,
    /// Apparent angular radius in radians.
    pub angular_radius: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaLight {
    /// Center position.
    pub position: Vec3,
    /// Local U axis vector.
    pub u: Vec3,
    /// Local V axis vector.
    pub v: Vec3,
    /// Emitted color.
    pub color: Vec3,
    /// Emission intensity multiplier.
    pub intensity: f64,
}

impl AreaLight {
    /// Samples a point on the rectangular area light.
    pub fn sample_point(&self, u: f64, v: f64) -> Vec3 {
        self.position + self.u * (u - 0.5) + self.v * (v - 0.5)
    }
}

/// Complete scene description used by the raytracer.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Sphere primitives.
    pub objects: Vec<Sphere>,
    /// Triangle primitives.
    pub triangles: Vec<Triangle>,
    /// Directional sun light.
    pub sun: DirectionalLight,
    /// Explicit area lights.
    pub area_lights: Vec<AreaLight>,
    /// Top sky color.
    pub sky_top: Vec3,
    /// Bottom sky color.
    pub sky_bottom: Vec3,
    /// Global exposure multiplier.
    pub exposure: f64,
    /// Participating medium configuration.
    pub volume: VolumetricMedium,
    /// Optional procedural environment map.
    pub hdri: Option<ProceduralEnvironment>,
    /// Solar elevation angle for environment logic.
    pub solar_elevation: f64,
}

impl Scene {
    /// Returns a hash signature of scene geometry for cache invalidation.
    pub fn geometry_signature(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.objects.len().hash(&mut hasher);
        self.triangles.len().hash(&mut hasher);

        for object in &self.objects {
            object.center.x.to_bits().hash(&mut hasher);
            object.center.y.to_bits().hash(&mut hasher);
            object.center.z.to_bits().hash(&mut hasher);
            object.radius.to_bits().hash(&mut hasher);
        }

        for triangle in &self.triangles {
            triangle.a.x.to_bits().hash(&mut hasher);
            triangle.a.y.to_bits().hash(&mut hasher);
            triangle.a.z.to_bits().hash(&mut hasher);
            triangle.b.x.to_bits().hash(&mut hasher);
            triangle.b.y.to_bits().hash(&mut hasher);
            triangle.b.z.to_bits().hash(&mut hasher);
            triangle.c.x.to_bits().hash(&mut hasher);
            triangle.c.y.to_bits().hash(&mut hasher);
            triangle.c.z.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Returns the closest hit record for a ray, if any.
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, t_min, closest) {
                closest = hit.distance;
                result = Some(hit);
            }
        }
        for triangle in &self.triangles {
            if let Some(hit) = triangle.hit(ray, t_min, closest) {
                closest = hit.distance;
                result = Some(hit);
            }
        }

        result
    }

    /// Returns true if any primitive blocks the ray within max_distance.
    pub fn is_occluded(&self, ray: &Ray, max_distance: f64) -> bool {
        for object in &self.objects {
            if object.hit(ray, EPSILON, max_distance).is_some() {
                return true;
            }
        }
        for triangle in &self.triangles {
            if triangle.hit(ray, EPSILON, max_distance).is_some() {
                return true;
            }
        }
        false
    }
}
