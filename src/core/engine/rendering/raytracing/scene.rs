use super::math::Vec3;
use super::primitives::{HitRecord, Ray, Sphere, Triangle, EPSILON};
use crate::core::engine::rendering::{
    environment::procedural::ProceduralEnvironment,
    effects::volumetric_effects::medium::VolumetricMedium,
};
use std::hash::{Hash, Hasher};

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

/// Scène complète prête à être rendue : objets, éclairage, environnement et volume.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Liste des sphères présentes dans la scène.
    pub objects: Vec<Sphere>,
    /// Liste des triangles présents dans la scène.
    pub triangles: Vec<Triangle>,
    /// Lumière directionnelle principale (soleil).
    pub sun: DirectionalLight,
    /// Lumières surfaciques.
    pub area_lights: Vec<AreaLight>,
    /// Couleur du ciel en haut `[r, g, b]`.
    pub sky_top: Vec3,
    /// Couleur du ciel en bas / horizon `[r, g, b]`.
    pub sky_bottom: Vec3,
    /// Facteur d'exposition global.
    pub exposure: f64,
    /// Médium volumétrique de la scène.
    pub volume: VolumetricMedium,
    /// Environnement HDRI procédural optionnel.
    pub hdri: Option<ProceduralEnvironment>,
    /// Élévation solaire en radians (utilisée par le rendu atmosphérique).
    pub solar_elevation: f64,
}

impl Scene {
    /// Calcule un hash de la géométrie (positions et rayons) pour détecter les changements.
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

    /// Retourne le `HitRecord` le plus proche le long du rayon, ou `None` si aucune intersection.
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

    /// Teste si un rayon est occulté dans la distance donnée (ombre portée).
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
