/// External physics configuration passed into the material builder.
///
/// This lets the same engine work for physically accurate simulations
/// (real values) **or** stylised/game rendering (tweaked values).
#[derive(Debug, Clone, Copy)]
pub struct PhysicsConfig {
    /// Index of refraction (e.g. 1.0 vacuum, 1.33 water, 1.52 glass, 2.42 diamond).
    pub ior: f64,
    /// Abbe number for chromatic dispersion (higher = less dispersion).
    pub dispersion_abbe: f64,
    /// Rayleigh scattering coefficient (atmosphere simulations).
    pub rayleigh_coefficient: f64,
    /// Mie scattering coefficient.
    pub mie_coefficient: f64,
    /// Mie scattering direction bias (g parameter, −1..1).
    pub mie_direction: f64,
    /// Absorption coefficient per channel (volumetric media).
    pub absorption: [f64; 3],
    /// Scattering coefficient per channel (volumetric media).
    pub scattering: [f64; 3],
    /// Phase function asymmetry for participating media (Henyey-Greenstein g).
    pub phase_asymmetry: f64,
    /// Gravitational lensing strength (0 = off, used for relativistic scenes).
    pub gravitational_lensing: f64,
    /// Doppler shift factor (fraction of c, for relativistic rendering).
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
