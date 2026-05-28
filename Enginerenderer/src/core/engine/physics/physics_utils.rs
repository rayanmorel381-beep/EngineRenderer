use crate::core::engine::rendering::raytracing::Vec3;

pub fn gravitational_force(mass_a: f64, mass_b: f64, delta: Vec3) -> Vec3 {
    let distance_squared = delta.length_squared().max(0.001);
    let force_magnitude = (mass_a * mass_b) / distance_squared;
    delta.normalize() * force_magnitude
}

pub fn orbital_stability(total_energy: f64, radius_hint: f64) -> f64 {
    (radius_hint / (1.0 + total_energy.abs()).sqrt()).clamp(0.0, 1.0)
}
