#[derive(Debug, Clone, Copy)]
pub struct PhysicsConfig {
    pub ior: f64,
    pub dispersion_abbe: f64,
    pub rayleigh_coefficient: f64,
    pub mie_coefficient: f64,
    pub mie_direction: f64,
    pub absorption: [f64; 3],
    pub scattering: [f64; 3],
    pub phase_asymmetry: f64,
    pub gravitational_lensing: f64,
    pub doppler_factor: f64,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            ior: 1.0,
            dispersion_abbe: 0.0,
            rayleigh_coefficient: 0.0,
            mie_coefficient: 0.0,
            mie_direction: 0.0,
            absorption: [0.0; 3],
            scattering: [0.0; 3],
            phase_asymmetry: 0.0,
            gravitational_lensing: 0.0,
            doppler_factor: 0.0,
        }
    }
}
