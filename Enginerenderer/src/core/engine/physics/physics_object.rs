use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct RigidBodyState {
    pub mass: f64,
    pub radius: f64,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl RigidBodyState {
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.mass * self.velocity.length_squared()
    }

    pub fn momentum_magnitude(&self) -> f64 {
        (self.velocity * self.mass).length()
    }
}
