use crate::core::engine::rendering::raytracing::{Material, Vec3};

use super::physics::PhysicsConfig;
use super::spectrum::Spectrum;

#[derive(Debug, Clone)]
pub struct MaterialBuilder {
    albedo: Vec3,
    emission: Vec3,
    roughness: f64,
    metallic: f64,
    reflectivity: f64,
    ambient_occlusion: f64,
    clearcoat: f64,
    sheen: Vec3,
    transmission: f64,
    ior: f64,
    subsurface: f64,
    anisotropy: f64,
    iridescence: f64,
    texture_weight: f64,
    normal_map_strength: f64,
    uv_scale: f64,
    physics: PhysicsConfig,
    spectrum: Option<Spectrum>,
}

impl Default for MaterialBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            albedo: Vec3::ZERO,
            emission: Vec3::ZERO,
            roughness: 0.5,
            metallic: 0.0,
            reflectivity: 0.0,
            ambient_occlusion: 1.0,
            clearcoat: 0.0,
            sheen: Vec3::ZERO,
            transmission: 0.0,
            ior: 1.0,
            subsurface: 0.0,
            anisotropy: 0.0,
            iridescence: 0.0,
            texture_weight: 0.0,
            normal_map_strength: 1.0,
            uv_scale: 1.0,
            physics: PhysicsConfig::default(),
            spectrum: None,
        }
    }

    // -- colour / albedo ----------------------------------------------------

    pub fn albedo_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.albedo = Vec3::new(r, g, b);
        self
    }

    pub fn albedo(mut self, v: Vec3) -> Self {
        self.albedo = v;
        self
    }

    pub fn albedo_spectrum(mut self, spec: Spectrum) -> Self {
        self.albedo = spec.to_rgb();
        self.spectrum = Some(spec);
        self
    }

    pub fn albedo_spectrum_custom(mut self, spec: Spectrum, xyz_to_rgb: [[f64; 3]; 3]) -> Self {
        self.albedo = spec.to_rgb_custom(xyz_to_rgb);
        self.spectrum = Some(spec);
        self
    }

    pub fn albedo_temperature(mut self, kelvin: f64, peak: f64) -> Self {
        let spec = Spectrum::black_body(kelvin, peak);
        self.albedo = spec.to_rgb();
        self.spectrum = Some(spec);
        self
    }

    // -- emission -----------------------------------------------------------

    pub fn emission_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.emission = Vec3::new(r, g, b);
        self
    }

    pub fn emission(mut self, v: Vec3) -> Self {
        self.emission = v;
        self
    }

    pub fn emission_spectrum(mut self, spec: Spectrum) -> Self {
        self.emission = spec.to_rgb();
        self
    }

    pub fn emission_temperature(mut self, kelvin: f64, peak: f64) -> Self {
        self.emission = Spectrum::black_body(kelvin, peak).to_rgb();
        self
    }

    // -- PBR surface --------------------------------------------------------

    pub fn roughness(mut self, v: f64) -> Self {
        self.roughness = v;
        self
    }

    pub fn metallic(mut self, v: f64) -> Self {
        self.metallic = v;
        self
    }

    pub fn reflectivity(mut self, v: f64) -> Self {
        self.reflectivity = v;
        self
    }

    pub fn ambient_occlusion(mut self, v: f64) -> Self {
        self.ambient_occlusion = v;
        self
    }

    pub fn clearcoat(mut self, v: f64) -> Self {
        self.clearcoat = v;
        self
    }

    pub fn sheen_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.sheen = Vec3::new(r, g, b);
        self
    }

    pub fn sheen(mut self, v: Vec3) -> Self {
        self.sheen = v;
        self
    }

    // -- transmission / refraction ------------------------------------------

    pub fn transmission(mut self, v: f64) -> Self {
        self.transmission = v;
        self
    }

    pub fn ior(mut self, v: f64) -> Self {
        self.ior = v;
        self
    }

    // -- advanced optics ----------------------------------------------------

    pub fn subsurface(mut self, v: f64) -> Self {
        self.subsurface = v;
        self
    }

    pub fn anisotropy(mut self, v: f64) -> Self {
        self.anisotropy = v;
        self
    }

    pub fn iridescence(mut self, v: f64) -> Self {
        self.iridescence = v;
        self
    }

    // -- texturing ----------------------------------------------------------

    pub fn texture_weight(mut self, v: f64) -> Self {
        self.texture_weight = v;
        self
    }

    pub fn normal_map_strength(mut self, v: f64) -> Self {
        self.normal_map_strength = v;
        self
    }

    pub fn uv_scale(mut self, v: f64) -> Self {
        self.uv_scale = v;
        self
    }

    // -- physics config (all external) --------------------------------------

    pub fn physics(mut self, cfg: PhysicsConfig) -> Self {
        self.physics = cfg;
        if cfg.ior > 0.0 {
            self.ior = cfg.ior;
        }
        self
    }

    pub fn physics_ior(mut self, v: f64) -> Self {
        self.physics.ior = v;
        self.ior = v;
        self
    }

    pub fn physics_dispersion(mut self, abbe: f64) -> Self {
        self.physics.dispersion_abbe = abbe;
        self
    }

    pub fn physics_scattering(mut self, rayleigh: f64, mie: f64, mie_dir: f64) -> Self {
        self.physics.rayleigh_coefficient = rayleigh;
        self.physics.mie_coefficient = mie;
        self.physics.mie_direction = mie_dir;
        self
    }

    pub fn physics_volume(mut self, absorption: [f64; 3], scattering: [f64; 3], phase_g: f64) -> Self {
        self.physics.absorption = absorption;
        self.physics.scattering = scattering;
        self.physics.phase_asymmetry = phase_g;
        self
    }

    pub fn physics_relativistic(mut self, grav_lensing: f64, doppler: f64) -> Self {
        self.physics.gravitational_lensing = grav_lensing;
        self.physics.doppler_factor = doppler;
        self
    }

    // -- accessors ----------------------------------------------------------

    pub fn get_physics(&self) -> &PhysicsConfig {
        &self.physics
    }

    pub fn get_spectrum(&self) -> Option<&Spectrum> {
        self.spectrum.as_ref()
    }

    // -- build --------------------------------------------------------------

    pub fn build(self) -> Material {
        Material::new(
            self.albedo,
            self.roughness,
            self.metallic,
            self.reflectivity,
            self.emission,
        )
        .with_layers(self.ambient_occlusion, self.clearcoat, self.sheen)
        .with_transmission(self.transmission, self.ior)
        .with_optics(self.subsurface, self.anisotropy, self.iridescence)
        .with_texturing(self.texture_weight, self.normal_map_strength, self.uv_scale)
    }
}
