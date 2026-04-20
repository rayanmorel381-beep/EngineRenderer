use crate::core::engine::{
    physics::{
        physics_object::RigidBodyState,
        physics_utils::{gravitational_force, orbital_stability},
    },
    rendering::raytracing::Vec3,
    scene::celestial::CelestialBodies,
};

#[derive(Debug, Clone)]
pub struct PhysicsManager {
    bodies: Vec<RigidBodyState>,
    stability_score: f64,
}

impl PhysicsManager {
    pub fn from_bodies(catalog: &CelestialBodies) -> Self {
        let mut manager = Self {
            bodies: Vec::new(),
            stability_score: 0.0,
        };
        manager.rebuild_from_bodies(catalog);
        manager
    }

    pub fn rebuild_from_bodies(&mut self, catalog: &CelestialBodies) {
        self.bodies = catalog
            .bodies()
            .iter()
            .map(|body| RigidBodyState {
                mass: body.mass,
                radius: body.radius,
                position: body.position,
                velocity: Vec3::ZERO,
            })
            .collect();

        let orbital = orbital_stability(self.total_kinetic_energy(), catalog.scene_radius());
        let momentum_factor = (self.total_momentum() / (1.0 + self.average_orbital_radius())).sqrt();
        let gravity_factor = self.net_gravity_measure().ln_1p();
        self.stability_score = (orbital * 0.68
            + (1.0 / (1.0 + momentum_factor)).clamp(0.0, 1.0) * 0.16
            + (1.0 / (1.0 + gravity_factor)).clamp(0.0, 1.0) * 0.16)
            .clamp(0.0, 1.0);
    }

    pub fn total_kinetic_energy(&self) -> f64 {
        self.bodies.iter().map(RigidBodyState::kinetic_energy).sum()
    }

    pub fn total_momentum(&self) -> f64 {
        self.bodies.iter().map(RigidBodyState::momentum_magnitude).sum()
    }

    pub fn average_orbital_radius(&self) -> f64 {
        if self.bodies.is_empty() {
            0.0
        } else {
            self.bodies
                .iter()
                .map(|body| body.position.length() + body.radius)
                .sum::<f64>()
                / self.bodies.len() as f64
        }
    }

    pub fn net_gravity_measure(&self) -> f64 {
        let mut measure = 0.0;
        for (index, body) in self.bodies.iter().enumerate() {
            for other in self.bodies.iter().skip(index + 1) {
                measure += gravitational_force(body.mass, other.mass, other.position - body.position).length();
            }
        }
        measure
    }

    pub fn stability_score(&self) -> f64 {
        self.stability_score
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }
}
