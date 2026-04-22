/// Extended physical material properties for advanced rendering effects.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsConfig {
    /// Index of refraction.
    pub ior: f64,
    /// Abbe number used for dispersion approximation.
    pub dispersion_abbe: f64,
    /// Rayleigh scattering intensity.
    pub rayleigh_coefficient: f64,
    /// Mie scattering intensity.
    pub mie_coefficient: f64,
    /// Preferred Mie scattering direction.
    pub mie_direction: f64,
    /// Per-channel absorption coefficient.
    pub absorption: [f64; 3],
    /// Per-channel scattering coefficient.
    pub scattering: [f64; 3],
    /// Phase function asymmetry parameter.
    pub phase_asymmetry: f64,
    /// Strength of gravitational lensing distortion.
    pub gravitational_lensing: f64,
    /// Doppler shift intensity factor.
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
