use crate::core::engine::rendering::{
    environment::procedural::ProceduralEnvironment,
    materials::material::MaterialLibrary,
    raytracing::{DirectionalLight, Material, Scene, Sphere, Vec3},
    effects::volumetric_effects::medium::VolumetricMedium,
};

/// Constante gravitationnelle simplifiée utilisée par la simulation N-corps.
pub const GRAVITY: f64 = 0.08;

/// Corps céleste simulé avec masse, rayon, état cinématique et matériau.
#[derive(Debug, Clone, Copy)]
pub struct CelestialBody {
    /// Masse du corps.
    pub mass: f64,
    /// Rayon du corps.
    pub radius: f64,
    /// Position actuelle du centre du corps.
    pub position: Vec3,
    /// Vitesse actuelle du corps.
    pub velocity: Vec3,
    /// Matériau de surface utilisé lors de la conversion en scène.
    pub material: Material,
}

/// Système N-corps avec intégration explicite simple.
#[derive(Debug, Clone)]
pub struct NBodySystem {
    bodies: Vec<CelestialBody>,
}

impl NBodySystem {
    /// Retourne une configuration de démonstration avec plusieurs corps en orbite.
    pub fn showcase() -> Self {
        Self {
            bodies: vec![
                CelestialBody {
                    mass: 1200.0,
                    radius: 1.6,
                    position: Vec3::new(0.0, 0.0, 0.0),
                    velocity: Vec3::ZERO,
                    material: MaterialLibrary::stellar_surface(),
                },
                CelestialBody {
                    mass: 5.0,
                    radius: 0.55,
                    position: Vec3::new(4.8, 0.3, 0.0),
                    velocity: Vec3::new(0.0, 0.0, 4.25),
                    material: MaterialLibrary::ocean_world(),
                },
                CelestialBody {
                    mass: 3.8,
                    radius: 0.42,
                    position: Vec3::new(-7.0, -0.25, 0.0),
                    velocity: Vec3::new(0.0, 0.0, -3.45),
                    material: MaterialLibrary::icy_world(),
                },
                CelestialBody {
                    mass: 6.4,
                    radius: 0.68,
                    position: Vec3::new(0.0, 0.15, 9.2),
                    velocity: Vec3::new(-3.15, 0.0, 0.0),
                    material: MaterialLibrary::rocky_world(Vec3::new(0.71, 0.42, 0.26)),
                },
                CelestialBody {
                    mass: 1.2,
                    radius: 0.24,
                    position: Vec3::new(5.9, 0.55, 0.0),
                    velocity: Vec3::new(0.0, 0.0, 5.65),
                    material: MaterialLibrary::metallic_moon(),
                },
            ],
        }
    }

    /// Retourne une vue immuable sur les corps simulés.
    pub fn bodies(&self) -> &[CelestialBody] {
        &self.bodies
    }

    /// Fait avancer la simulation pendant `total_time` découpé en `substeps` sous-pas.
    pub fn advance(&mut self, total_time: f64, substeps: u32) {
        let steps = substeps.max(1);
        let delta_time = total_time / steps as f64;
        for _ in 0..steps {
            self.step(delta_time);
        }
    }

    fn step(&mut self, delta_time: f64) {
        let body_count = self.bodies.len();
        let mut accelerations = vec![Vec3::ZERO; body_count];

        for i in 0..body_count {
            for j in (i + 1)..body_count {
                let delta = self.bodies[j].position - self.bodies[i].position;
                let distance_squared = delta.length_squared().max(0.35);
                let distance = distance_squared.sqrt();
                let direction = delta / distance;
                let acceleration_i = direction * (GRAVITY * self.bodies[j].mass / distance_squared);
                let acceleration_j = direction * (GRAVITY * self.bodies[i].mass / distance_squared);

                accelerations[i] += acceleration_i;
                accelerations[j] = accelerations[j] - acceleration_j;
            }
        }

        for (body, acceleration) in self.bodies.iter_mut().zip(accelerations.into_iter()) {
            body.velocity += acceleration * delta_time;
            body.position += body.velocity * delta_time;
        }
    }

    /// Calcule le centre de masse pondéré du système.
    pub fn scene_center(&self) -> Vec3 {
        let mut weighted_sum = Vec3::ZERO;
        let mut total_mass = 0.0;

        for body in &self.bodies {
            weighted_sum += body.position * body.mass;
            total_mass += body.mass;
        }

        if total_mass <= f64::EPSILON {
            Vec3::ZERO
        } else {
            weighted_sum / total_mass
        }
    }

    /// Calcule le rayon englobant tous les corps depuis le centre de masse.
    pub fn scene_radius(&self) -> f64 {
        let center = self.scene_center();
        self.bodies
            .iter()
            .map(|body| (body.position - center).length() + body.radius)
            .fold(1.0, f64::max)
    }

    /// Convertit l'état simulé courant en `Scene` prête au rendu.
    pub fn to_scene(&self) -> Scene {
        let objects = self
            .bodies
            .iter()
            .map(|body| Sphere {
                center: body.position,
                radius: body.radius,
                material: body.material,
            })
            .collect();

        Scene {
            objects,
            triangles: Vec::new(),
            sun: DirectionalLight {
                direction: Vec3::new(-0.65, -0.35, -1.0).normalize(),
                color: Vec3::new(1.0, 0.96, 0.90),
                intensity: 1.45,
                angular_radius: 0.03,
            },
            area_lights: Vec::new(),
            sky_top: Vec3::new(0.015, 0.020, 0.050),
            sky_bottom: Vec3::new(0.001, 0.001, 0.006),
            exposure: 1.45,
            volume: VolumetricMedium::cinematic_nebula().with_density_multiplier(0.9),
            hdri: Some(ProceduralEnvironment::cinematic_space()),
            solar_elevation: 0.48,
        }
    }
}
