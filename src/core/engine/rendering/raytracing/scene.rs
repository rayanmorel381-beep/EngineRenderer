use super::math::Vec3;
use super::primitives::{HitRecord, Ray, Sphere, Triangle, EPSILON};
use crate::core::engine::rendering::{
    environment::procedural::ProceduralEnvironment,
    effects::volumetric_effects::medium::VolumetricMedium,
};

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f64,
    pub angular_radius: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaLight {
    pub position: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub color: Vec3,
    pub intensity: f64,
}

impl AreaLight {
    pub fn sample_point(&self, u: f64, v: f64) -> Vec3 {
        self.position + self.u * (u - 0.5) + self.v * (v - 0.5)
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub objects: Vec<Sphere>,
    pub triangles: Vec<Triangle>,
    pub sun: DirectionalLight,
    pub area_lights: Vec<AreaLight>,
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
    pub exposure: f64,
    pub volume: VolumetricMedium,
    pub hdri: Option<ProceduralEnvironment>,
    pub solar_elevation: f64,
}

impl Scene {
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
