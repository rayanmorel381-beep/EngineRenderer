use crate::core::engine::rendering::{
    environment::procedural::ProceduralEnvironment,
    materials::material::MaterialLibrary,
    raytracing::{DirectionalLight, Material, Scene, Sphere, Vec3},
    effects::volumetric_effects::medium::VolumetricMedium,
};

#[derive(Debug, Clone, Copy)]
/// Physical and rendering properties for one celestial body.
pub struct CelestialBody {
    /// Relative mass used for center-of-mass computations.
    pub mass: f64,
    /// Body radius.
    pub radius: f64,
    /// Body center position.
    pub position: Vec3,
    /// Surface material.
    pub material: Material,
}

#[derive(Debug, Clone)]
/// Collection of celestial bodies used by showcase scenes.
pub struct CelestialBodies {
    bodies: Vec<CelestialBody>,
}

impl CelestialBodies {
    /// Builds the default showcase body catalog.
    pub fn showcase() -> Self {
        Self {
            bodies: vec![
                CelestialBody {
                    mass: 1200.0,
                    radius: 1.6,
                    position: Vec3::new(0.0, 0.0, 0.0),
                    material: MaterialLibrary::stellar_surface(),
                },
                CelestialBody {
                    mass: 5.0,
                    radius: 0.55,
                    position: Vec3::new(4.8, 0.3, 0.0),
                    material: MaterialLibrary::ocean_world(),
                },
                CelestialBody {
                    mass: 3.8,
                    radius: 0.42,
                    position: Vec3::new(-7.0, -0.25, 0.0),
                    material: MaterialLibrary::icy_world(),
                },
                CelestialBody {
                    mass: 6.4,
                    radius: 0.68,
                    position: Vec3::new(0.0, 0.15, 9.2),
                    material: MaterialLibrary::rocky_world(Vec3::new(0.71, 0.42, 0.26)),
                },
                CelestialBody {
                    mass: 1.2,
                    radius: 0.24,
                    position: Vec3::new(5.9, 0.55, 0.0),
                    material: MaterialLibrary::metallic_moon(),
                },
            ],
        }
    }

    /// Returns all celestial bodies.
    pub fn bodies(&self) -> &[CelestialBody] {
        &self.bodies
    }

    /// Returns the mass-weighted scene center.
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

    /// Returns the radial extent of the full body set.
    pub fn scene_radius(&self) -> f64 {
        let center = self.scene_center();
        self.bodies
            .iter()
            .map(|body| (body.position - center).length() + body.radius)
            .fold(1.0, f64::max)
    }

    /// Converts the body catalog into a renderable scene.
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
